use quote::ToTokens;
use syn::{
    Expr,
    spanned::Spanned,
    visit::{self, Visit},
};

pub(super) fn tokens(node: &impl ToTokens) -> String {
    node.to_token_stream()
        .to_string()
        .split_whitespace()
        .collect()
}
struct Checks<'a> {
    source: &'a crate::Source,
    output: &'a mut Vec<crate::Finding>,
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
}

impl<'ast> Visit<'ast> for Checks<'_> {
    fn visit_block(&mut self, node: &'ast syn::Block) {
        let end = node.span().end();
        let start = self.source.offset(node.span());
        let end = self.source.position_offset(end.line, end.column);
        let text: String = self.source.masked[start..end].split_whitespace().collect();

        if text.contains("BufWriter::new(")
            && (text.contains(".write(") || text.contains(".write_all("))
            && !text.contains(".flush(")
        {
            self.emit(
                "buffered-write-flush-policy",
                node,
                "Buffered write has no explicit flush in this block.",
            );
        }

        for (index, statement) in node.stmts.iter().enumerate() {
            if let syn::Stmt::Local(local) = statement {
                let syn::Pat::Ident(binding) = &local.pat else {
                    continue;
                };

                let Some(initialise) = &local.init else {
                    continue;
                };

                let name = binding.ident.to_string();
                let initializer = tokens(&initialise.expr);
                let later = node.stmts[index + 1..]
                    .iter()
                    .map(tokens)
                    .collect::<Vec<_>>()
                    .join(";");

                if (initializer.contains(".lock(")
                    || initializer.contains(".read(")
                    || initializer.contains(".write("))
                    && let Some(await_position) = later.find(".await")
                    && !later[..await_position].contains(&format!("drop({name})"))
                {
                    self.emit("lock-held-await",local,"Lock-shaped local has no explicit drop before await; type and liveness are unresolved.");
                }

                let clone = format!("{name}.clone()");

                if let Some(position) = later.rfind(&clone) {
                    let suffix = &later[position + clone.len()..];

                    if !suffix.contains(&name)
                        && later.matches(&clone).count() == 1
                        && !super::precision::repeated_context(&node.stmts[index + 1..])
                    {
                        self.emit("clone-before-move",local,"Local is cloned at its last lexical use; control flow and Drop may prevent a move.");
                    }
                }

                if let Some(syn::Stmt::Expr(Expr::MethodCall(call), None)) = node.stmts.last()
                    && ["as_ptr", "as_mut_ptr"]
                        .iter()
                        .any(|member| call.method == *member)
                    && tokens(&call.receiver) == name
                {
                    self.emit(
                        "raw-pointer-lifetime",
                        call,
                        "Returned pointer refers to a local owner; check its lifetime.",
                    );
                }
            }
        }

        visit::visit_block(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        let callee = tokens(&node.func);

        if callee.ends_with("Regex::new")
            && matches!(
                node.args.first(),
                Some(Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(_),
                    ..
                }))
            )
        {
            self.emit(
                "regex-per-call-literal",
                node,
                "Literal regex is compiled in executable code.",
            );
        }

        visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "to_string"
            && matches!(
                &*node.receiver,
                Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(_),
                    ..
                })
            )
        {
            self.emit(
                "efficiency-extra-string-literal-owned",
                node,
                "String literal is converted to an owned String.",
            );
        }

        visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if node.path.is_ident("format") {
            use syn::{Token, parse::Parser, punctuated::Punctuated};

            if let Ok(arguments) =
                Punctuated::<Expr, Token![,]>::parse_terminated.parse2(node.tokens.clone())
                && let Some(Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(literal),
                    ..
                })) = arguments.first()
            {
                if arguments.len() == 1 && !literal.value().contains('{') {
                    self.emit(
                        "efficiency-extra-format-literal-only",
                        node,
                        "Formatting a literal without substitution creates an owned string",
                    );
                }

                if arguments.len() == 2 && literal.value() == "{}" {
                    self.emit(
                        "efficiency-extra-format-only-value",
                        node,
                        "Formatting uses only one Display value.",
                    );
                }
            }
        }

        visit::visit_macro(self, node);
    }

    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        let offset = self.source.offset(node.span());
        let prefix = &self.source.text[..offset];
        let nearby = prefix.lines().rev().take(4).collect::<Vec<_>>().join(" ");

        if !nearby.to_ascii_lowercase().contains("safety:") {
            self.emit(
                "unsafe-missing-contract",
                node,
                "Unsafe block lacks a nearby SAFETY comment.",
            );
        }

        visit::visit_expr_unsafe(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if node.sig.unsafety.is_some() {
            let offset = self.source.offset(node.span());
            let prefix = &self.source.text[..offset];

            if !prefix
                .lines()
                .rev()
                .take(8)
                .any(|line| line.to_ascii_lowercase().contains("safety"))
                && !tokens(
                    &node
                        .attrs
                        .iter()
                        .map(ToTokens::to_token_stream)
                        .collect::<proc_macro2::TokenStream>(),
                )
                .to_ascii_lowercase()
                .contains("safety")
            {
                self.emit(
                    "unsafe-missing-contract",
                    node,
                    "Unsafe function lacks visible safety contract documentation",
                );
            }
        }

        visit::visit_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        if node.unsafety.is_some() {
            let offset = self.source.offset(node.span());

            if !self.source.text[..offset]
                .lines()
                .rev()
                .take(4)
                .any(|line| line.to_ascii_lowercase().contains("safety"))
            {
                self.emit(
                    "unsafe-missing-contract",
                    node,
                    "Unsafe implementation lacks nearby safety rationale",
                );
            }
        }

        visit::visit_item_impl(self, node);
    }
}

pub(super) fn analyze(source: &crate::Source, output: &mut Vec<crate::Finding>) {
    Checks { source, output }.visit_file(&source.ast);
}
