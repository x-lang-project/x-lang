use std::collections;

use syn::visit::{self, Visit};

use super::tokens;

pub(super) fn analyze(source: &crate::Source, findings: &mut Vec<crate::Finding>) {
    Functions { source, findings }.visit_file(&source.ast);
}
struct Functions<'a> {
    source: &'a crate::Source,
    findings: &'a mut Vec<crate::Finding>,
}
#[derive(Default)]
struct Count {
    decisions: usize,
    locals: usize,
    roots: collections::HashSet<String>,
}
impl<'ast> Visit<'ast> for Count {
    fn visit_expr_if(&mut self, expression_value: &'ast syn::ExprIf) {
        self.decisions += 1;
        visit::visit_expr_if(self, expression_value);
    }

    fn visit_expr_match(&mut self, expression_value: &'ast syn::ExprMatch) {
        self.decisions += expression_value.arms.len().saturating_sub(1);
        visit::visit_expr_match(self, expression_value);
    }

    fn visit_expr_for_loop(&mut self, expression_value: &'ast syn::ExprForLoop) {
        self.decisions += 1;
        visit::visit_expr_for_loop(self, expression_value);
    }

    fn visit_expr_while(&mut self, expression_value: &'ast syn::ExprWhile) {
        self.decisions += 1;
        visit::visit_expr_while(self, expression_value);
    }

    fn visit_expr_binary(&mut self, expression_value: &'ast syn::ExprBinary) {
        if matches!(expression_value.op, syn::BinOp::And(_) | syn::BinOp::Or(_)) {
            self.decisions += 1;
        }

        visit::visit_expr_binary(self, expression_value);
    }

    fn visit_local(&mut self, error: &'ast syn::Local) {
        self.locals += 1;
        visit::visit_local(self, error);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path.segments.len() >= 2 {
            self.roots.insert(path.segments[0].ident.to_string());
        }

        visit::visit_path(self, path);
    }

    fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {}

    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}
}

impl Functions<'_> {
    fn function(
        &mut self,
        signature: &syn::Signature,
        visibility: &syn::Visibility,
        second_value: &syn::Block,
    ) {
        let mut character = Count::default();
        character.visit_block(second_value);
        for (hit, identifier, message) in [
            (
                character.decisions >= 12,
                "decision-complexity-outlier",
                "Function contains at least twelve syntactic decisions excluding nested closures/functions.",
            ),
            (
                character.locals >= 16,
                "local-binding-density",
                "Function declares at least sixteen locals; this counts declarations rather than live values.",
            ),
            (
                character.roots.len() >= 6,
                "multi-purpose-helper-fanout",
                "Function uses qualified paths from at least six root names; module identities are unresolved.",
            ),
        ] {
            if hit {
                crate::ReviewSink {
                    source: self.source,
                    findings: self.findings,
                }
                .at_span(identifier, signature.ident.span(), message);
            }
        }

        if matches!(visibility, syn::Visibility::Inherited)
            && second_value.stmts.len() == 1
            && let syn::Stmt::Expr(syn::Expr::Call(call), _) = &second_value.stmts[0]
        {
            let arguments: Vec<_> = signature
                .inputs
                .iter()
                .filter_map(|first_value| {
                    if let syn::FnArg::Typed(type_value) = first_value {
                        if let syn::Pat::Ident(pointer) = &*type_value.pat {
                            Some(pointer.ident.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            if arguments.len() == signature.inputs.len()
                && call.args.iter().map(tokens).collect::<Vec<_>>() == arguments
                && signature.ident != tokens(&call.func)
            {
                crate::ReviewSink {
                    source: self.source,
                    findings: self.findings,
                }
                .at_span(
                    "unnecessary-pass-through-wrapper",
                    signature.ident.span(),
                    "Private helper forwards unchanged arguments to another call.",
                );
            }
        }
    }
}

impl<'ast> Visit<'ast> for Functions<'_> {
    fn visit_item_fn(&mut self, callee: &'ast syn::ItemFn) {
        self.function(&callee.sig, &callee.vis, &callee.block);
        visit::visit_item_fn(self, callee);
    }

    fn visit_impl_item_fn(&mut self, callee: &'ast syn::ImplItemFn) {
        self.function(&callee.sig, &callee.vis, &callee.block);
        visit::visit_impl_item_fn(self, callee);
    }
}
