use std::collections::BTreeSet;

use syn::{
    Expr,
    spanned::Spanned,
    visit::{self, Visit},
};

pub(super) fn repeated_context(statements: &[syn::Stmt]) -> bool {
    struct Repeated(bool);
    impl<'ast> Visit<'ast> for Repeated {
        fn visit_expr_for_loop(&mut self, _: &'ast syn::ExprForLoop) {
            self.0 = true;
        }

        fn visit_expr_while(&mut self, _: &'ast syn::ExprWhile) {
            self.0 = true;
        }

        fn visit_expr_loop(&mut self, _: &'ast syn::ExprLoop) {
            self.0 = true;
        }

        fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {
            self.0 = true;
        }
    }
    let mut repeated = Repeated(false);

    for statement in statements {
        repeated.visit_stmt(statement);
    }

    repeated.0
}

fn vec_type(value_type: &syn::Type) -> bool {
    matches!(value_type, syn::Type::Path(current_path) if current_path.path.segments.last().is_some_and(|text| text.ident == "Vec"))
}

fn vec_initializer(expression: &Expr) -> bool {
    match expression {
        Expr::Macro(member) => member.mac.path.is_ident("vec"),
        Expr::Call(call_expression) => {
            matches!(&*call_expression.func, Expr::Path(current_path) if current_path.path.segments.iter().any(|text| text.ident == "Vec"))
        }
        _ => false,
    }
}
struct Checks<'a> {
    source: &'a crate::Source,
    output: &'a mut Vec<crate::Finding>,
    vectors: BTreeSet<String>,
    guards: BTreeSet<String>,
    loop_depth: usize,
}
impl Checks<'_> {
    fn emit(&mut self, identifier: &str, node: &impl Spanned, message: &str) {
        self.source.emit(
            self.output,
            identifier,
            self.source.offset(node.span()),
            message,
        );
    }

    fn loop_body(&mut self, block: &syn::Block) {
        self.loop_depth += 1;
        self.visit_block(block);
        self.loop_depth -= 1;
    }

    fn guarded_names(expression: &Expr) -> BTreeSet<String> {
        let mut names = BTreeSet::new();

        if let Expr::Binary(binary_expression) = expression {
            if matches!(binary_expression.op, syn::BinOp::And(_)) {
                names.extend(Self::guarded_names(&binary_expression.left));
                names.extend(Self::guarded_names(&binary_expression.right));
            } else if let Expr::MethodCall(character) = &*binary_expression.left
                && character.method == "len"
                && matches!(&*binary_expression.right, Expr::Lit(literal) if matches!(&literal.lit, syn::Lit::Int(index) if index.base10_parse::<usize>().is_ok_and(|count| match binary_expression.op { syn::BinOp::Gt(_) => true, syn::BinOp::Ge(_) | syn::BinOp::Eq(_) => count > 0, _ => false })))
            {
                names.insert(super::ast_checks::tokens(&character.receiver));
            }
        } else if let Expr::Unary(u) = expression
            && matches!(u.op, syn::UnOp::Not(_))
            && let Expr::MethodCall(character) = &*u.expr
            && character.method == "is_empty"
        {
            names.insert(super::ast_checks::tokens(&character.receiver));
        }

        names
    }
}

impl<'ast> Visit<'ast> for Checks<'_> {
    fn visit_block(&mut self, node: &'ast syn::Block) {
        let previous = self.vectors.clone();

        for statement in &node.stmts {
            if let syn::Stmt::Local(local) = statement {
                let (name, typed_vec) = match &local.pat {
                    syn::Pat::Ident(pointer) => (Some(pointer.ident.to_string()), false),
                    syn::Pat::Type(pointer) => match &*pointer.pat {
                        syn::Pat::Ident(identifier) => {
                            (Some(identifier.ident.to_string()), vec_type(&pointer.ty))
                        }
                        _ => (None, false),
                    },
                    _ => (None, false),
                };

                if let Some(name) = name {
                    self.vectors.remove(&name);

                    if typed_vec
                        || local
                            .init
                            .as_ref()
                            .is_some_and(|index| vec_initializer(&index.expr))
                    {
                        self.vectors.insert(name);
                    }
                }
            }

            self.visit_stmt(statement);
        }
        self.vectors = previous;
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.visit_expr(&node.expr);
        let previous = self.vectors.clone();

        if let syn::Pat::Ident(binding) = &*node.pat {
            self.vectors.remove(&binding.ident.to_string());
        }

        self.loop_body(&node.body);
        self.vectors = previous;
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.loop_depth += 1;
        self.visit_expr(&node.cond);
        self.visit_block(&node.body);
        self.loop_depth -= 1;
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        self.loop_body(&node.body);
    }

    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}

    fn visit_item(&mut self, _: &'ast syn::Item) {}

    fn visit_expr_reference(&mut self, node: &'ast syn::ExprReference) {
        if matches!(&*node.expr, Expr::Reference(_)) {
            self.emit("redundant-borrow-layers", node, "Nested reference expression; confirm whether the callee requires this reference depth");
        }

        visit::visit_expr_reference(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if self.loop_depth > 0
            && super::ast_checks::tokens(&node.func).ends_with("Regex::new")
            && matches!(node.args.first(), Some(Expr::Lit(literal)) if matches!(literal.lit, syn::Lit::Str(_)))
        {
            self.emit("regex-in-loop", node, "Literal regex constructor executes within a loop; review caching this invariant pattern");
        }

        visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "len"
            && let Expr::MethodCall(character) = &*node.receiver
            && character.method == "collect"
            && character.turbofish.as_ref().is_some_and(|arguments| {
                arguments.args
                    .iter()
                    .any(|argument| matches!(argument, syn::GenericArgument::Type(value_type) if vec_type(value_type)))
            })
        {
            self.emit(
                "collect-length-only",
                node,
                "Explicit Vec collection used only for its length; consider iterator count",
            );
        }

        if self.loop_depth > 0
            && node.method == "contains"
            && self
                .vectors
                .contains(&super::ast_checks::tokens(&node.receiver))
        {
            self.emit("linear-membership-loop", node, "Contains on an explicitly Vec-shaped local inside a loop; review list size and repeated search cost");
        }

        visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        self.visit_expr(&node.left);
        let previous = self.guards.clone();

        if matches!(node.op, syn::BinOp::And(_)) {
            self.guards.extend(Self::guarded_names(&node.left));
        }

        self.visit_expr(&node.right);
        self.guards = previous;
    }

    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.visit_expr(&node.cond);
        let previous = self.guards.clone();
        self.guards.extend(Self::guarded_names(&node.cond));
        self.visit_block(&node.then_branch);
        self.guards = previous;

        if let Some((_, expression)) = &node.else_branch {
            self.visit_expr(expression);
        }
    }

    fn visit_expr_index(&mut self, node: &'ast syn::ExprIndex) {
        if matches!(&*node.index, Expr::Lit(literal) if matches!(&literal.lit, syn::Lit::Int(position) if position.base10_parse::<usize>().is_ok_and(|count| count == 0)))
            && !self.guards.contains(&super::ast_checks::tokens(&node.expr))
        {
            self.emit("zero-length-index", node, "Direct index-zero access without a recognised local emptiness guard; upstream invariants may guarantee safety");
        }

        visit::visit_expr_index(self, node);
    }
}

pub(super) fn analyze(source: &crate::Source, output: &mut Vec<crate::Finding>) {
    analyze_items(source, output, &source.ast.items);
}

fn analyze_items(source: &crate::Source, output: &mut Vec<crate::Finding>, items: &[syn::Item]) {
    for item in items {
        let mut checks = Checks {
            source,
            output,
            vectors: BTreeSet::new(),
            guards: BTreeSet::new(),
            loop_depth: 0,
        };

        match item {
            syn::Item::Fn(function) => checks.visit_block(&function.block),
            syn::Item::Impl(index) => {
                for item in &index.items {
                    if let syn::ImplItem::Fn(function) = item {
                        checks.visit_block(&function.block);
                    }
                }
            }
            syn::Item::Mod(module) => {
                if let Some((_, items)) = &module.content {
                    analyze_items(source, checks.output, items);
                }
            }
            syn::Item::Trait(trait_item) => {
                for item in &trait_item.items {
                    if let syn::TraitItem::Fn(function) = item
                        && let Some(block) = &function.default
                    {
                        checks.visit_block(block);
                    }
                }
            }
            _ => {}
        }
    }
}
