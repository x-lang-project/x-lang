use std::collections::{BTreeMap, BTreeSet};

use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

use crate::{Finding, Source};

mod precision;

struct Function {
    source: usize,
    offset: usize,
    column: usize,
    test: bool,
    name: String,
    body: String,
    normalized: String,
    reference_shape: String,
    substantive: bool,
}
struct Collector<'a> {
    functions: &'a mut Vec<Function>,
    source: usize,
    source_data: &'a Source,
}
impl Collector<'_> {
    fn add(&mut self, signature: &syn::Signature, block: &syn::Block) {
        let body = block.to_token_stream().to_string();
        let mut bindings = precision::Bindings { names: vec![] };
        bindings.visit_signature(signature);
        bindings.visit_block(block);
        let normalized = precision::normalize(block.to_token_stream(), &bindings.names);
        self.functions.push(Function {
            source: self.source,
            offset: signature.span().start().line,
            column: signature.span().start().column,
            test: self.source_data.scope_at(
                signature.span().start().line,
                signature.span().start().column + 1,
            ) == crate::scope::Scope::Test,
            name: signature.ident.to_string(),
            body,
            normalized,
            reference_shape: precision::reference_shape(signature),
            substantive: !precision::delegate(block),
        });
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.add(&node.sig, &node.block);
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.add(&node.sig, &node.block);
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if let Some(block) = &node.default {
            self.add(&node.sig, block);
        }

        visit::visit_trait_item_fn(self, node);
    }
}

fn groups<'a>(
    functions: &'a [Function],
    key: impl Fn(&'a Function) -> &'a str,
    minimum: usize,
) -> Vec<Vec<&'a Function>> {
    let mut groups = BTreeMap::<(&str, &str, bool), Vec<&Function>>::new();

    for function in functions {
        if function.body.split_whitespace().count() >= minimum
            && (minimum == 0 || function.substantive)
        {
            let shape = if minimum == 0 {
                ""
            } else {
                &function.reference_shape
            };

            groups
                .entry((key(function), shape, function.test))
                .or_default()
                .push(function);
        }
    }

    groups
        .into_values()
        .filter(|group| group.len() > 1)
        .collect()
}

pub fn analyze(sources: &[Source], output: &mut Vec<Finding>) {
    let mut functions = vec![];

    for (index, source) in sources.iter().enumerate() {
        Collector {
            functions: &mut functions,
            source: index,
            source_data: source,
        }
        .visit_file(&source.ast);
    }

    for (identifier, grouped) in [
        (
            "repeated-function-name",
            groups(&functions, |field| &field.name, 0),
        ),
        (
            "exact-function-body",
            groups(&functions, |field| &field.body, 24),
        ),
        (
            "renamed-function-body",
            groups(&functions, |field| &field.normalized, 24),
        ),
    ] {
        for group in grouped {
            if identifier == "renamed-function-body"
                && group.iter().all(|field| field.body == group[0].body)
            {
                continue;
            }

            let locations = group
                .iter()
                .take(20)
                .map(|field| {
                    format!(
                        "{}:{} ({})",
                        sources[field.source].path, field.offset, field.name
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            let first = group[0];
            sources[first.source].emit(
                output,
                identifier,
                sources[first.source].position_offset(first.offset, first.column),
                format!("{} matching functions: {locations}.", group.len()),
            );
        }
    }

    let mut index = BTreeMap::<(bool, String), Vec<usize>>::new();

    for (position, function) in functions
        .iter()
        .enumerate()
        .filter(|(_, field)| field.substantive && field.body.split_whitespace().count() >= 24)
    {
        let words = function.body.split_whitespace().collect::<Vec<_>>();
        let shingles: BTreeSet<_> = words.windows(5).map(|window| window.join(" ")).collect();

        for shingle in shingles {
            index
                .entry((function.test, shingle))
                .or_default()
                .push(position);
        }
    }

    let mut compared = BTreeSet::new();
    let skipped = index.values().filter(|members| members.len() > 64).count();

    if skipped > 0
        && let Some(source) = sources.first()
    {
        source.emit(output, "similarity-coverage-limit", 0, format!("Skipped {skipped} shingle buckets above 64 functions; exact and renamed groups were checked."));
    }

    for members in index.values().filter(|members| members.len() <= 64) {
        for (current_position, first_value) in members.iter().enumerate() {
            for second_value in members.iter().skip(current_position + 1) {
                if !compared.insert((*first_value, *second_value)) {
                    continue;
                }

                let first_value = &functions[*first_value];
                let second_value = &functions[*second_value];

                if first_value.test != second_value.test
                    || first_value.reference_shape != second_value.reference_shape
                    || first_value.body == second_value.body
                    || first_value.normalized == second_value.normalized
                {
                    continue;
                }

                let left: BTreeSet<_> = first_value.body.split_whitespace().collect();
                let right: BTreeSet<_> = second_value.body.split_whitespace().collect();
                let score =
                    left.intersection(&right).count() as f64 / left.union(&right).count() as f64;

                if score >= 0.9 {
                    sources[first_value.source].emit(
                        output,
                        "similar-function-body",
                        sources[first_value.source]
                            .position_offset(first_value.offset, first_value.column),
                        format!(
                            "Token-set similarity {:.0}% with {}:{}; order and types may differ.",
                            score * 100.0,
                            sources[second_value.source].path,
                            second_value.offset
                        ),
                    );
                }
            }
        }
    }

    let mut files = BTreeMap::<(&str, bool), Vec<&Source>>::new();

    for source in sources {
        if precision::substantive_file(&source.ast) {
            files
                .entry((&source.text, source.is_test_file()))
                .or_default()
                .push(source);
        }
    }

    for group in files.values().filter(|group| group.len() > 1) {
        group[0].emit(
            output,
            "duplicate-file",
            0,
            format!(
                "{} byte-identical files: {}",
                group.len(),
                group
                    .iter()
                    .map(|current_text| current_text.path.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
    }
}
