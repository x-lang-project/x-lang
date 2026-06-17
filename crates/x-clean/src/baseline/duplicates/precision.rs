use quote::ToTokens;
use syn::visit::{self, Visit};

pub(super) struct Bindings {
    pub(super) names: Vec<String>,
}
impl<'ast> Visit<'ast> for Bindings {
    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        let name = node.ident.to_string();

        if !self.names.contains(&name) {
            self.names.push(name);
        }
    }
}

pub(super) fn normalize(tokens: proc_macro2::TokenStream, names: &[String]) -> String {
    let mut result = Vec::new();
    let mut member = false;

    for token in tokens {
        match token {
            proc_macro2::TokenTree::Ident(identifier) => {
                let name = identifier.to_string();
                result.push(
                    if !member && let Some(index) = names.iter().position(|count| count == &name) {
                        format!("__local_{index}")
                    } else {
                        name
                    },
                );
                member = false;
            }
            proc_macro2::TokenTree::Group(group) => {
                result.push(format!(
                    "{:?}({})",
                    group.delimiter(),
                    normalize(group.stream(), names)
                ));
                member = false;
            }
            proc_macro2::TokenTree::Punct(punct) => {
                member = matches!(punct.as_char(), '.' | ':');
                result.push(punct.to_string());
            }
            literal => {
                result.push(literal.to_string());
                member = false;
            }
        }
    }

    result.join(" ")
}

// Identical forwarding calls can implement different traits or receiver types.
pub(super) fn delegate(block: &syn::Block) -> bool {
    let statements: Vec<_> = block
        .stmts
        .iter()
        .filter(|statement| !matches!(statement, syn::Stmt::Item(syn::Item::Use(_))))
        .collect();

    match statements.as_slice() {
        [syn::Stmt::Expr(expression, _)] => forwarding_expression(expression),
        [syn::Stmt::Macro(_)] => true,
        _ => false,
    }
}

fn forwarding_expression(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Call(call) => call.args.iter().all(simple_argument),
        syn::Expr::MethodCall(call) => {
            simple_argument(&call.receiver) && call.args.iter().all(simple_argument)
        }
        syn::Expr::Return(value) => value.expr.as_deref().is_some_and(forwarding_expression),
        syn::Expr::Try(value) => forwarding_expression(&value.expr),
        syn::Expr::Await(value) => forwarding_expression(&value.base),
        syn::Expr::Paren(value) => forwarding_expression(&value.expr),
        syn::Expr::Group(value) => forwarding_expression(&value.expr),
        syn::Expr::Macro(_) => true,
        _ => false,
    }
}

fn simple_argument(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Path(_) | syn::Expr::Lit(_) => true,
        syn::Expr::Reference(value) => simple_argument(&value.expr),
        syn::Expr::Field(value) => simple_argument(&value.base),
        syn::Expr::Paren(value) => simple_argument(&value.expr),
        syn::Expr::Group(value) => simple_argument(&value.expr),
        syn::Expr::Unary(value) => simple_argument(&value.expr),
        syn::Expr::Struct(value) => {
            value
                .fields
                .iter()
                .all(|field| simple_argument(&field.expr))
                && value.rest.as_deref().is_none_or(simple_argument)
        }
        syn::Expr::Call(_) | syn::Expr::MethodCall(_) => forwarding_expression(expression),
        _ => false,
    }
}

pub(super) fn reference_shape(signature: &syn::Signature) -> String {
    struct References(String);
    impl<'ast> Visit<'ast> for References {
        fn visit_type_reference(&mut self, node: &'ast syn::TypeReference) {
            self.0
                .push(if node.mutability.is_some() { 'm' } else { 's' });
            visit::visit_type_reference(self, node);
        }

        fn visit_type_ptr(&mut self, node: &'ast syn::TypePtr) {
            self.0
                .push(if node.mutability.is_some() { 'M' } else { 'S' });
            visit::visit_type_ptr(self, node);
        }

        fn visit_receiver(&mut self, node: &'ast syn::Receiver) {
            self.0
                .push(if node.mutability.is_some() { 'm' } else { 's' });
            visit::visit_receiver(self, node);
        }
    }
    let mut references = References(String::new());
    references.visit_signature(signature);

    references.0
}

pub(super) fn substantive_file(file: &syn::File) -> bool {
    file.items.iter().any(|item| match item {
        syn::Item::Fn(function) => {
            !delegate(&function.block)
                && function
                    .block
                    .to_token_stream()
                    .to_string()
                    .split_whitespace()
                    .count()
                    >= 24
        }
        syn::Item::Use(_) | syn::Item::ExternCrate(_) | syn::Item::Macro(_) => false,
        syn::Item::Mod(module) if module.content.is_none() => false,
        _ => {
            item.to_token_stream()
                .to_string()
                .split_whitespace()
                .count()
                >= 24
        }
    })
}
