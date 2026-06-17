use std::collections;

mod inventory;
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};

use super::{docs, tokens};

pub(super) use inventory::analyze;
struct Documentation<'a> {
    source: &'a crate::Source,
    findings: &'a mut Vec<crate::Finding>,
}
impl Documentation<'_> {
    fn flag(&mut self, identifier: &str, span: proc_macro2::Span, message: &str) {
        crate::ReviewSink {
            source: self.source,
            findings: self.findings,
        }
        .at_span(identifier, span, message);
    }

    fn item(
        &mut self,
        attributes: &[syn::Attribute],
        visibility: &syn::Visibility,
        name: &syn::Ident,
    ) {
        let definition = docs(attributes);

        if matches!(visibility, syn::Visibility::Public(_))
            && definition.trim().is_empty()
            && !attributes
                .iter()
                .any(|first_value| first_value.path().is_ident("doc"))
        {
            self.flag(
                "public-item-undocumented",
                name.span(),
                "Public item lacks attached rustdoc; external reachability is unresolved.",
            );
        }

        if !definition.is_empty() {
            let compact = definition.to_lowercase().replace([' ', '_', '.', '\n'], "");
            let item = name.to_string().to_lowercase().replace('_', "");

            if compact == item {
                self.flag(
                    "boilerplate-name-only-doc",
                    name.span(),
                    "Documentation only restates the normalised item name.",
                );
            }
        }

        if attributes
            .iter()
            .any(|first_value| first_value.path().is_ident("doc"))
            && definition.trim().is_empty()
        {
            self.flag(
                "clarity-extra-empty-doc",
                name.span(),
                "Attached documentation has no content.",
            );
        }

        let mut seen = collections::HashSet::new();

        for line in definition.lines() {
            for (hit, identifier, message) in [
                (
                    !line.is_ascii(),
                    "clarity-extra-nonascii-doc",
                    "Documentation contains non-ASCII text; review language/character consistency.",
                ),
                (
                    !line.trim().is_empty() && !seen.insert(line.trim()),
                    "clarity-extra-duplicate-doc-line",
                    "Identical nonempty documentation line appears twice.",
                ),
                (
                    line.ends_with(' ') || line.ends_with('\t'),
                    "clarity-extra-doc-trailing-space",
                    "Documentation line has trailing whitespace.",
                ),
                (
                    line.contains('\t'),
                    "clarity-extra-doc-tab",
                    "Documentation contains a literal tab.",
                ),
                (
                    line.chars().count() > 120,
                    "clarity-extra-doc-long-line",
                    "Documentation line exceeds 120 characters.",
                ),
            ] {
                if hit {
                    self.flag(identifier, name.span(), message);
                }
            }
        }

        if definition
            .lines()
            .filter(|left_value| left_value.trim().starts_with("```"))
            .count()
            % 2
            == 1
        {
            self.flag(
                "clarity-extra-doc-unclosed-fence",
                name.span(),
                "Documentation contains an odd number of fenced-code delimiters.",
            );
        }

        if definition.contains("```") && definition.contains(".unwrap(") {
            self.flag(
                "clarity-extra-doc-example-unwrap",
                name.span(),
                "Documentation example calls unwrap.",
            );
        }
    }

    fn function(
        &mut self,
        attributes: &[syn::Attribute],
        visibility: &syn::Visibility,
        signature: &syn::Signature,
        block: &syn::Block,
    ) {
        self.item(attributes, visibility, &signature.ident);
        let definition = docs(attributes);
        let parameters: collections::HashSet<_> = signature
            .inputs
            .iter()
            .filter_map(|pointer| {
                if let syn::FnArg::Typed(type_value) = pointer {
                    if let syn::Pat::Ident(index) = &*type_value.pat {
                        Some(index.ident.to_string())
                    } else {
                        None
                    }
                } else {
                    Some("self".into())
                }
            })
            .collect();
        let rx = regex::Regex::new(r"(?m)^\s*[-*]\s*`([A-Za-z_][A-Za-z_0-9]*)`\s*:").unwrap();

        if definition.contains("# Arguments") || definition.contains("# Parameters") {
            for character in rx.captures_iter(&definition) {
                if !parameters.contains(&character[1]) {
                    self.flag(
                        "stale-parameter-doc-name",
                        signature.ident.span(),
                        &format!(
                            "Structured parameter documentation names {}, absent from signature.",
                            &character[1]
                        ),
                    );
                }
            }
        }

        if matches!(visibility, syn::Visibility::Public(_))
            && !definition.is_empty()
            && tokens(block).contains("Err (")
            && !definition.contains("# Errors")
        {
            self.flag(
                "undocumented-behavior-branch",
                signature.ident.span(),
                "Documented public function explicitly constructs Err but has no Errors section.",
            );
        }
    }
}

impl<'ast> Visit<'ast> for Documentation<'_> {
    fn visit_item_trait(&mut self, index: &'ast syn::ItemTrait) {
        self.item(&index.attrs, &index.vis, &index.ident);
        visit::visit_item_trait(self, index);
    }

    fn visit_item_union(&mut self, index: &'ast syn::ItemUnion) {
        self.item(&index.attrs, &index.vis, &index.ident);
        visit::visit_item_union(self, index);
    }

    fn visit_item_static(&mut self, index: &'ast syn::ItemStatic) {
        self.item(&index.attrs, &index.vis, &index.ident);
        visit::visit_item_static(self, index);
    }

    fn visit_field(&mut self, index: &'ast syn::Field) {
        if let Some(name) = &index.ident {
            self.item(&index.attrs, &index.vis, name);
        }

        visit::visit_field(self, index);
    }

    fn visit_item_fn(&mut self, index: &'ast syn::ItemFn) {
        self.function(&index.attrs, &index.vis, &index.sig, &index.block);
        visit::visit_item_fn(self, index);
    }

    fn visit_impl_item_fn(&mut self, index: &'ast syn::ImplItemFn) {
        self.function(&index.attrs, &index.vis, &index.sig, &index.block);
        visit::visit_impl_item_fn(self, index);
    }

    fn visit_item_struct(&mut self, index: &'ast syn::ItemStruct) {
        self.item(&index.attrs, &index.vis, &index.ident);
        visit::visit_item_struct(self, index);
    }

    fn visit_item_enum(&mut self, index: &'ast syn::ItemEnum) {
        self.item(&index.attrs, &index.vis, &index.ident);
        visit::visit_item_enum(self, index);
    }

    fn visit_item_mod(&mut self, index: &'ast syn::ItemMod) {
        self.item(&index.attrs, &index.vis, &index.ident);
        visit::visit_item_mod(self, index);
    }

    fn visit_item_type(&mut self, index: &'ast syn::ItemType) {
        self.item(&index.attrs, &index.vis, &index.ident);
        visit::visit_item_type(self, index);
    }

    fn visit_item_const(&mut self, index: &'ast syn::ItemConst) {
        self.item(&index.attrs, &index.vis, &index.ident);
        if let syn::Expr::Lit(literal) = &*index.expr
            && let syn::Lit::Int(count) = &literal.lit
        {
            let line = index.span().start().line;
            let previous = self
                .source
                .text
                .lines()
                .nth(line.saturating_sub(2))
                .unwrap_or("");
            let rx = regex::Regex::new(r"(?i)//\s*(?:limit|count|version)\s*[:=]\s*(\d+)").unwrap();

            if let Some(character) = rx.captures(previous)
                && character[1].parse::<u64>().ok() != count.base10_parse::<u64>().ok()
            {
                self.flag("comment-contradicts-literal",index.ident.span(),"Immediately preceding limit/count/version comment disagrees with numeric constant.");
            }
        }

        visit::visit_item_const(self, index);
    }
}
