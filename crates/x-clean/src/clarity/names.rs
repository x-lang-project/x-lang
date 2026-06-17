mod locals;
use std::collections::{HashMap, HashSet};

use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};

use super::tokens;

pub(super) fn analyze(source: &crate::Source, findings: &mut Vec<crate::Finding>) {
    let mut writer = Names { source, findings };
    writer.visit_file(&source.ast);
    let identifiers: HashSet<_> = source
        .masked
        .split(|character: char| !character.is_alphanumeric() && character != '_')
        .collect();

    for words in [
        ["cfg", "config", "configuration"],
        ["diag", "diagnostic", "diagnostics"],
    ] {
        if words
            .iter()
            .filter(|writer| identifiers.contains(**writer))
            .count()
            >= 2
        {
            source.emit(
                writer.findings,
                "abbreviation-variant",
                0,
                format!(
                    "Abbreviation family uses multiple spellings: {}.",
                    words.join(" / ")
                ),
            );
        }
    }

    for (first_value, second_value) in [
        ("source_file", "input_file"),
        ("module_id", "module_key"),
        ("output_file", "destination_file"),
    ] {
        if identifiers.contains(first_value) && identifiers.contains(second_value) {
            source.emit(
                writer.findings,
                "inconsistent-role-vocabulary",
                0,
                format!(
                    "Configured role vocabulary pair co-occurs: {first_value} / {second_value}."
                ),
            );
        }
    }
}
struct Names<'a> {
    source: &'a crate::Source,
    findings: &'a mut Vec<crate::Finding>,
}
impl Names<'_> {
    fn flag(&mut self, identifier: &str, span: proc_macro2::Span, message: &str) {
        crate::ReviewSink {
            source: self.source,
            findings: self.findings,
        }
        .at_span(identifier, span, message);
    }

    fn name(&mut self, name: &syn::Ident, local: bool) {
        let text = name.to_string();
        let bare = text.trim_start_matches("r#");

        for (hit, identifier, message) in [
            (
                text.starts_with("r#"),
                "clarity-extra-raw-name",
                "Raw identifier requires keyword escaping.",
            ),
            (
                !text.is_ascii(),
                "clarity-extra-unicode-name",
                "Identifier includes non-ASCII characters; review visual distinction.",
            ),
            (
                bare.len() > 1 && bare.ends_with('_'),
                "clarity-extra-trailing-underscore",
                "Name ends with an underscore.",
            ),
            (
                bare.contains("__"),
                "clarity-extra-repeated-name-separator",
                "Name contains consecutive underscore separators.",
            ),
            (
                bare.len() > 48,
                "clarity-extra-long-name",
                "Identifier exceeds 48 characters.",
            ),
            (
                bare.chars()
                    .filter(|character| character.is_ascii_digit())
                    .count()
                    >= 4,
                "clarity-extra-digit-heavy-name",
                "Identifier has at least four numeric characters.",
            ),
            (
                local
                    && bare.len() > 1
                    && bare
                        .chars()
                        .all(|character| character.is_ascii_uppercase() || character == '_'),
                "clarity-extra-uppercase-local",
                "Local binding uses constant-style uppercase spelling.",
            ),
            (
                local
                    && bare.contains('_')
                    && bare.chars().any(|character| character.is_ascii_uppercase()),
                "clarity-extra-mixed-case-local",
                "Local binding mixes underscores and uppercase letters.",
            ),
        ] {
            if hit {
                self.flag(identifier, name.span(), message);
            }
        }
    }

    fn type_name(&mut self, pattern: &syn::Pat, value_type: &syn::Type) {
        if let syn::Pat::Ident(pointer) = pattern {
            let count = pointer.ident.to_string();
            let type_value = tokens(value_type);

            if (count.ends_with("_path") || count.ends_with("_paths"))
                && ["u8", "u16", "u32", "u64", "usize", "i32", "bool"]
                    .contains(&type_value.as_str())
            {
                self.flag(
                    "name-type-disagreement",
                    pointer.span(),
                    "Path-role name has an explicitly numeric or Boolean type.",
                );
            }

            if (count.starts_with("is_") || count.starts_with("has_"))
                && ["u8", "u32", "usize", "String", "i32"].contains(&type_value.as_str())
            {
                self.flag(
                    "clarity-extra-boolean-role-type",
                    pointer.span(),
                    "Boolean-role prefix has an explicitly non-Boolean type.",
                );
            }
        }
    }
}

impl<'ast> Visit<'ast> for Names<'_> {
    fn visit_pat_ident(&mut self, pointer: &'ast syn::PatIdent) {
        self.name(&pointer.ident, true);
        visit::visit_pat_ident(self, pointer);
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        self.name(&function.sig.ident, false);
        visit::visit_item_fn(self, function);
    }

    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        self.name(&function.sig.ident, false);
        visit::visit_impl_item_fn(self, function);
    }

    fn visit_generic_param(&mut self, pointer: &'ast syn::GenericParam) {
        if let syn::GenericParam::Type(type_value) = pointer
            && type_value.ident.to_string().len() > 20
        {
            self.flag(
                "clarity-extra-long-generic-name",
                type_value.ident.span(),
                "Type parameter name exceeds twenty characters.",
            );
        }

        visit::visit_generic_param(self, pointer);
    }

    fn visit_pat_type(&mut self, pointer: &'ast syn::PatType) {
        self.type_name(&pointer.pat, &pointer.ty);
        visit::visit_pat_type(self, pointer);
    }

    fn visit_expr_unary(&mut self, expression_value: &'ast syn::ExprUnary) {
        if matches!(expression_value.op, syn::UnOp::Not(_))
            && let syn::Expr::Path(current_path) = &*expression_value.expr
            && let Some(count) = current_path.path.get_ident()
        {
            let count = count.to_string();

            if count.starts_with("not_")
                || count.starts_with("disable_")
                || count.starts_with("no_")
            {
                self.flag(
                    "negative-boolean-name",
                    expression_value.span(),
                    "Negative-role name is negated again at this use.",
                );
            }
        }

        visit::visit_expr_unary(self, expression_value);
    }

    fn visit_item_impl(&mut self, index: &'ast syn::ItemImpl) {
        if index.trait_.is_none()
            && let syn::Type::Path(type_value) = &*index.self_ty
            && let Some(owner) = type_value.path.segments.last()
        {
            let o = owner.ident.to_string().to_lowercase();

            for item in &index.items {
                if let syn::ImplItem::Fn(function) = item
                    && function
                        .sig
                        .ident
                        .to_string()
                        .to_lowercase()
                        .split('_')
                        .any(|text| text == o)
                {
                    self.flag(
                        "redundant-owner-name",
                        function.sig.ident.span(),
                        "Inherent method name repeats its owning type name.",
                    );
                }
            }
        }

        visit::visit_item_impl(self, index);
    }

    fn visit_block(&mut self, second_value: &'ast syn::Block) {
        self.check_block(second_value);
    }
}
