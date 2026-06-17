mod catalog;
mod control;
mod docs;
mod duplication;
mod metrics;
mod names;

use quote::ToTokens;
use syn::visit::Visit;

pub fn rules() -> Vec<crate::Rule> {
    catalog::rules()
}

pub fn analyze(source: &crate::Source, findings: &mut Vec<crate::Finding>) {
    names::analyze(source, findings);
    docs::analyze(source, findings);
    control::Walker {
        source,
        findings,
        depth: 0,
    }
    .visit_file(&source.ast);
    metrics::analyze(source, findings);
}

pub fn analyze_project(sources: &[crate::Source], findings: &mut Vec<crate::Finding>) {
    duplication::analyze(sources, findings);
}

pub(super) fn tokens(node: &impl ToTokens) -> String {
    node.to_token_stream().to_string()
}

pub(super) fn docs(attributes: &[syn::Attribute]) -> String {
    attributes
        .iter()
        .filter_map(|first_value| {
            if !first_value.path().is_ident("doc") {
                return None;
            }

            if let syn::Meta::NameValue(element) = &first_value.meta
                && let syn::Expr::Lit(element) = &element.value
                && let syn::Lit::Str(element) = &element.lit
            {
                return Some(element.value());
            }

            None
        })
        .collect::<Vec<_>>()
        .join("\n")
}
