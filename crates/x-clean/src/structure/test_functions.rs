use super::*;

#[derive(Clone)]
pub(super) struct Function {
    pub name: String,
    pub body: String,
    pub code: String,
    pub offset: usize,
    pub test: bool,
    pub assertion: bool,
    pub unchecked_branch: bool,
    pub attributes: Vec<syn::Attribute>,
}
pub(super) fn functions(source: &Source) -> Vec<Function> {
    struct Collector<'a> {
        source: &'a Source,
        result: Vec<Function>,
        in_test: bool,
    }
    impl<'ast> Visit<'ast> for Collector<'_> {
        fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
            let old = self.in_test;
            self.in_test |= module.ident == "tests"
                || module
                    .attrs
                    .iter()
                    .any(|first_value| compact(first_value).contains("cfg(test)"));
            syn::visit::visit_item_mod(self, module);
            self.in_test = old;
        }

        fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
            let body = compact(&function.block);
            let code = crate::masking::mask(&body).replace(' ', "");
            let assertion = visible_check(&function.block)
                || !matches!(function.sig.output, syn::ReturnType::Default);
            self.result.push(Function {
                name: function.sig.ident.to_string(),
                body,
                code,
                offset: self.source.offset(function.span()),
                test: has_attribute(&function.attrs, "test"),
                assertion,
                unchecked_branch: unchecked_branch(&function.block),
                attributes: function.attrs.clone(),
            });
            syn::visit::visit_item_fn(self, function);
        }
    }
    let mut collector = Collector {
        source,
        result: Vec::new(),
        in_test: false,
    };
    collector.visit_file(&source.ast);

    collector.result
}

fn check_macro(macro_node: &syn::Macro) -> bool {
    macro_node.path.segments.last().is_some_and(|segment| {
        let name = segment.ident.to_string();

        name == "assert"
            || name == "assert_eq"
            || name == "assert_ne"
            || name.starts_with("debug_assert")
    })
}

fn visible_check(block: &syn::Block) -> bool {
    struct Checks(bool);
    impl<'ast> Visit<'ast> for Checks {
        fn visit_macro(&mut self, node: &'ast syn::Macro) {
            self.0 |= check_macro(node);
        }

        fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
            self.0 |= node.method == "expect";
            syn::visit::visit_expr_method_call(self, node);
        }

        fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
            self.0 |= matches!(&*node.func, syn::Expr::Path(current_path) if current_path.path.is_ident("verify"));
            syn::visit::visit_expr_call(self, node);
        }

        fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {}
    }
    let mut checks = Checks(false);
    checks.visit_block(block);

    checks.0
}

fn checked_expression(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Macro(member) => check_macro(&member.mac),
        syn::Expr::MethodCall(member) => member.method == "expect",
        syn::Expr::Call(call_expression) => {
            matches!(&*call_expression.func, syn::Expr::Path(current_path) if current_path.path.is_ident("verify"))
        }
        syn::Expr::Block(second_value) => guaranteed_check(&second_value.block),
        syn::Expr::If(index) => {
            guaranteed_check(&index.then_branch)
                && index
                    .else_branch
                    .as_ref()
                    .is_some_and(|(_, expression_value)| checked_expression(expression_value))
        }
        _ => false,
    }
}

fn guaranteed_check(block: &syn::Block) -> bool {
    block.stmts.iter().any(|statement| match statement {
        syn::Stmt::Macro(member) => check_macro(&member.mac),
        syn::Stmt::Expr(expression_value, _) => checked_expression(expression_value),
        syn::Stmt::Local(left_value) => left_value
            .init
            .as_ref()
            .is_some_and(|index| checked_expression(&index.expr)),
        _ => false,
    })
}

fn unchecked_branch(block: &syn::Block) -> bool {
    if guaranteed_check(block) {
        return false;
    }

    struct Partial(bool);
    impl<'ast> Visit<'ast> for Partial {
        fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
            let then_checked = guaranteed_check(&node.then_branch);
            let else_checked = node
                .else_branch
                .as_ref()
                .is_some_and(|(_, expression_value)| checked_expression(expression_value));
            self.0 |= then_checked != else_checked;
            syn::visit::visit_expr_if(self, node);
        }

        fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}

        fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {}
    }
    let mut partial = Partial(false);
    partial.visit_block(block);

    partial.0
}
