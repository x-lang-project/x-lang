mod items;
mod project;
mod registry;
mod test_functions;
mod tests;

use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::{Finding, Rule, Source};

pub fn rules() -> Vec<Rule> {
    registry::rules()
}

pub fn analyze(source: &Source, findings: &mut Vec<Finding>) {
    let mut visitor = items::Inspector { source, findings };
    visitor.visit_file(&source.ast);
    tests::analyze(source, visitor.findings);
}

pub fn analyze_project(sources: &[Source], findings: &mut Vec<Finding>) {
    project::analyze(sources, findings);
}

pub(super) fn tokens(value: &impl ToTokens) -> String {
    value.to_token_stream().to_string()
}

pub(super) fn compact(value: &impl ToTokens) -> String {
    tokens(value).replace(' ', "")
}

pub(super) fn is_public(visibility: &syn::Visibility) -> bool {
    matches!(visibility, syn::Visibility::Public(_))
}

pub(super) fn has_attribute(attributes: &[syn::Attribute], name: &str) -> bool {
    attributes
        .iter()
        .any(|first_value| first_value.path().is_ident(name))
}
