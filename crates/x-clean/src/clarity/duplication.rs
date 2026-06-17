use std::collections::HashMap;

use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};

use super::tokens;

mod statements;

fn literal(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Lit(_) => true,
        syn::Expr::Tuple(type_value) => type_value.elems.iter().all(literal),
        syn::Expr::Unary(u) => matches!(u.op, syn::UnOp::Neg(_)) && literal(&u.expr),
        _ => false,
    }
}

pub(super) fn analyze(sources: &[crate::Source], findings: &mut Vec<crate::Finding>) {
    statements::analyze(sources, findings);
    let mut tables = HashMap::new();
    let mut dispatch = HashMap::new();

    for source in sources {
        let mut writer = Dup {
            source,
            findings,
            tables: &mut tables,
            dispatch: &mut dispatch,
        };
        writer.visit_file(&source.ast);
    }
}
struct DuplicateSite<'a> {
    key: String,
    rule: &'a str,
    span: proc_macro2::Span,
}
struct Dup<'a> {
    source: &'a crate::Source,
    findings: &'a mut Vec<crate::Finding>,
    tables: &'a mut HashMap<String, String>,
    dispatch: &'a mut HashMap<String, String>,
}
impl Dup<'_> {
    fn repeat(
        sink: crate::ReviewSink<'_>,
        map: &mut HashMap<String, String>,
        site: DuplicateSite<'_>,
    ) {
        let DuplicateSite { key, rule, span } = site;
        let start = span.start();
        let key = format!(
            "{:?}|{key}",
            sink.source.scope_at(start.line, start.column + 1)
        );

        if let Some(first) = map.get(&key) {
            sink.at_span(
                rule,
                span,
                format!("Repeated syntax also occurs in {first}."),
            );
        } else {
            map.insert(key, format!("{}:{}", sink.source.path, span.start().line));
        }
    }

    fn expressions(&mut self, second_value: &syn::Block) {
        struct Expressions {
            seen: HashMap<String, proc_macro2::Span>,
            duplicates: Vec<proc_macro2::Span>,
        }
        impl<'a> Visit<'a> for Expressions {
            fn visit_expr(&mut self, expression: &'a syn::Expr) {
                let key = tokens(expression);

                if key.len() >= 45
                    && matches!(
                        expression,
                        syn::Expr::Binary(_) | syn::Expr::Call(_) | syn::Expr::MethodCall(_)
                    )
                    && self.seen.insert(key, expression.span()).is_some()
                {
                    self.duplicates.push(expression.span());
                }

                visit::visit_expr(self, expression);
            }

            fn visit_item_fn(&mut self, _: &'a syn::ItemFn) {}
        }
        let mut value = Expressions {
            seen: HashMap::new(),
            duplicates: vec![],
        };
        value.visit_block(second_value);

        for span in value.duplicates {
            crate::ReviewSink { source: self.source, findings: self.findings }.at_span("repeated-complex-expression", span, "Function repeats a call/binary expression with at least 45 normalised characters; effects are unresolved.");
        }
    }
}

impl<'ast> Visit<'ast> for Dup<'_> {
    fn visit_expr_array(&mut self, first_value: &'ast syn::ExprArray) {
        if first_value.elems.len() >= 8 && first_value.elems.iter().all(literal) {
            Self::repeat(
                crate::ReviewSink {
                    source: self.source,
                    findings: self.findings,
                },
                self.tables,
                DuplicateSite {
                    key: tokens(first_value),
                    rule: "duplicate-literal-table",
                    span: first_value.span(),
                },
            );
        }

        visit::visit_expr_array(self, first_value);
    }

    fn visit_expr_match(&mut self, member: &'ast syn::ExprMatch) {
        if member.arms.len() >= 8
            && member
                .arms
                .iter()
                .all(|first_value| first_value.guard.is_none() && literal(&first_value.body))
        {
            let key = member
                .arms
                .iter()
                .map(|first_value| {
                    format!(
                        "{}=>{}",
                        tokens(&first_value.pat),
                        tokens(&first_value.body)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            Self::repeat(
                crate::ReviewSink {
                    source: self.source,
                    findings: self.findings,
                },
                self.tables,
                DuplicateSite {
                    key,
                    rule: "duplicate-literal-table",
                    span: member.span(),
                },
            );
        }

        if member.arms.len() >= 3 {
            let key = format!(
                "{}|{}",
                tokens(&member.expr),
                member
                    .arms
                    .iter()
                    .map(|first_value| tokens(&first_value.pat))
                    .collect::<Vec<_>>()
                    .join("|")
            );
            Self::repeat(
                crate::ReviewSink {
                    source: self.source,
                    findings: self.findings,
                },
                self.dispatch,
                DuplicateSite {
                    key,
                    rule: "parallel-match-dispatch",
                    span: member.span(),
                },
            );
        }

        visit::visit_expr_match(self, member);
    }

    fn visit_expr_struct(&mut self, text: &'ast syn::ExprStruct) {
        let mut values: HashMap<String, Vec<String>> = HashMap::new();

        for field in &text.fields {
            if field.colon_token.is_some() {
                values
                    .entry(tokens(&field.expr))
                    .or_default()
                    .push(tokens(&field.member));
            }
        }

        for (value, fields) in values {
            if fields.len() >= 2 && fields.iter().any(|field| field == &value) {
                crate::ReviewSink { source: self.source, findings: self.findings }.at_span("copy-paste-field-outlier", text.span(), format!(
                        "Fields {} share value {value}, including a matching-name field; review asymmetric mapping.",
                        fields.join(", ")
                    ));
            }
        }

        visit::visit_expr_struct(self, text);
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        self.expressions(&function.block);
        visit::visit_item_fn(self, function);
    }

    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        self.expressions(&function.block);
        visit::visit_impl_item_fn(self, function);
    }
}
