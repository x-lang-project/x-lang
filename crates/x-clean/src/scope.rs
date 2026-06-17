use std::collections::BTreeMap;

use serde::Serialize;
use syn::{
    Attribute, Meta, Token, parse::Parser, punctuated::Punctuated, spanned::Spanned, visit::Visit,
};

use crate::{Finding, Report};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Source,
    Test,
}

#[derive(Debug, Serialize)]
pub struct ScopeSummary {
    pub findings: usize,
    pub formatting_candidates: usize,
    pub review_candidates: usize,
    pub affected_files: usize,
    pub by_rule: BTreeMap<String, usize>,
}

impl ScopeSummary {
    pub(crate) fn from_findings(findings: &[Finding], scope: Scope) -> Self {
        let mut summary = Self {
            findings: 0,
            formatting_candidates: 0,
            review_candidates: 0,
            affected_files: crate::reporting::affected_files(findings, scope),
            by_rule: BTreeMap::new(),
        };

        for finding in findings.iter().filter(|finding| finding.scope == scope) {
            summary.findings += 1;
            if crate::reporting::layout(&finding.rule) {
                summary.formatting_candidates += 1;
            } else {
                summary.review_candidates += 1;
            }
            *summary.by_rule.entry(finding.rule.clone()).or_default() += 1;
        }

        summary
    }
}

#[derive(Default)]
pub(crate) struct TestScopes {
    all: bool,
    explicit_file: bool,
    ranges: Vec<(proc_macro2::LineColumn, proc_macro2::LineColumn)>,
}

fn test_path(path: &str) -> bool {
    let path = path.replace('\\', "/");
    let filename = path.rsplit('/').next().unwrap_or(&path);

    path.split('/').any(|part| part == "tests")
        || filename == "tests.rs"
        || filename.ends_with("_tests.rs")
}

// Only a provably test-only cfg marks a test region.
fn can_exist_without_test(meta: &Meta) -> bool {
    match meta {
        Meta::Path(path) if path.is_ident("test") => false,
        Meta::List(list) => {
            let Ok(parts) =
                Punctuated::<Meta, Token![,]>::parse_terminated.parse2(list.tokens.clone())
            else {
                return true;
            };

            if list.path.is_ident("all") {
                parts.iter().all(can_exist_without_test)
            } else if list.path.is_ident("any") {
                parts.iter().any(can_exist_without_test)
            } else if list.path.is_ident("not") && parts.len() == 1 {
                // Unknown predicates are not safely negatable. Handle only test expressions.
                let part = &parts[0];

                known_without_test(part).is_none_or(|value| !value)
            } else {
                true
            }
        }
        _ => true,
    }
}

fn known_without_test(meta: &Meta) -> Option<bool> {
    match meta {
        Meta::Path(path) if path.is_ident("test") => Some(false),
        Meta::List(list) if list.path.is_ident("not") => {
            let parts = Punctuated::<Meta, Token![,]>::parse_terminated
                .parse2(list.tokens.clone())
                .ok()?;

            if parts.len() == 1 {
                known_without_test(&parts[0]).map(|value| !value)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn mentions_test(meta: &Meta) -> bool {
    match meta {
        Meta::Path(path) => path.is_ident("test"),
        Meta::List(list) => Punctuated::<Meta, Token![,]>::parse_terminated
            .parse2(list.tokens.clone())
            .is_ok_and(|parts| parts.iter().any(mentions_test)),
        Meta::NameValue(_) => false,
    }
}

fn test_only(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute
            .path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "test")
            || (attribute.path().is_ident("cfg")
                && attribute
                    .parse_args::<Meta>()
                    .is_ok_and(|meta| mentions_test(&meta) && !can_exist_without_test(&meta)))
    })
}

impl TestScopes {
    pub(crate) fn is_all(&self) -> bool {
        self.all
    }

    pub(crate) fn mark_all(&mut self) {
        self.all = true;
    }

    pub(crate) fn explicit_file(&self) -> bool {
        self.explicit_file
    }

    pub(crate) fn clear_inferred(&mut self) {
        self.all = self.explicit_file;
    }

    pub(crate) fn new(path: &str, ast: &syn::File) -> Self {
        let mut scopes = Self {
            all: test_path(path) || test_only(&ast.attrs),
            explicit_file: test_only(&ast.attrs),
            ranges: vec![],
        };
        scopes.visit_file(ast);

        scopes
    }

    pub(crate) fn at(&self, line: usize, column: usize) -> Scope {
        if self.all {
            Scope::Test
        } else {
            self.ast_at(line, column)
        }
    }

    pub(crate) fn ast_at(&self, line: usize, column: usize) -> Scope {
        let point = (line, column.saturating_sub(1));

        if self.explicit_file
            || self.ranges.iter().any(|(start, end)| {
                (start.line, start.column) <= point && point <= (end.line, end.column)
            })
        {
            Scope::Test
        } else {
            Scope::Source
        }
    }

    fn record(&mut self, span: proc_macro2::Span) {
        self.ranges.push((span.start(), span.end()));
    }
}

impl<'ast> Visit<'ast> for TestScopes {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let attributes: &[Attribute] = match item {
            syn::Item::Const(item) => &item.attrs,
            syn::Item::Enum(item) => &item.attrs,
            syn::Item::ExternCrate(item) => &item.attrs,
            syn::Item::Fn(item) => &item.attrs,
            syn::Item::ForeignMod(item) => &item.attrs,
            syn::Item::Impl(item) => &item.attrs,
            syn::Item::Macro(item) => &item.attrs,
            syn::Item::Mod(item) => &item.attrs,
            syn::Item::Static(item) => &item.attrs,
            syn::Item::Struct(item) => &item.attrs,
            syn::Item::Trait(item) => &item.attrs,
            syn::Item::TraitAlias(item) => &item.attrs,
            syn::Item::Type(item) => &item.attrs,
            syn::Item::Union(item) => &item.attrs,
            syn::Item::Use(item) => &item.attrs,
            _ => &[],
        };

        if test_only(attributes) {
            self.record(item.span());
        } else {
            syn::visit::visit_item(self, item);
        }
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        if test_only(&item.attrs) {
            self.record(item.span());
        } else {
            syn::visit::visit_impl_item_fn(self, item);
        }
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        if test_only(&item.attrs) {
            self.record(item.span());
        } else {
            syn::visit::visit_trait_item_fn(self, item);
        }
    }
}

pub fn render_report(report: &Report) -> String {
    crate::reporting::render(report)
}
