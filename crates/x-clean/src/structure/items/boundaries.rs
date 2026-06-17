use super::*;

impl Inspector<'_> {
    pub(super) fn inspect_attribute(&mut self, attribute: &syn::Attribute) {
        self.attributes(std::slice::from_ref(attribute));
        syn::visit::visit_attribute(self, attribute);
    }

    pub(super) fn inspect_item_function(&mut self, function: &syn::ItemFn) {
        self.signature(&function.sig, &function.vis);
        if is_public(&function.vis)
            && compact(&function.sig.output).contains("Result<")
            && !has_attribute(&function.attrs, "must_use")
        {
            self.hit(
                "structure-extra-public-result-no-must-use",
                function,
                "Public Result function lacks custom must_use guidance (Result itself is must_use)",
            );
        }

        syn::visit::visit_item_fn(self, function);
    }

    pub(super) fn inspect_impl_item_function(&mut self, count: &syn::ImplItemFn) {
        self.signature(&count.sig, &count.vis);
        syn::visit::visit_impl_item_fn(self, count);
    }

    pub(super) fn inspect_item_static(&mut self, static_item: &syn::ItemStatic) {
        if is_public(&static_item.vis)
            && matches!(static_item.mutability, syn::StaticMutability::Mut(_))
        {
            self.hit(
                "structure-extra-public-static-mut",
                static_item,
                "Public mutable static exposes global unsafe state",
            );
        }

        syn::visit::visit_item_static(self, static_item);
    }

    pub(super) fn inspect_item_foreign_mod(&mut self, foreign_module: &syn::ItemForeignMod) {
        if !has_attribute(&foreign_module.attrs, "link") {
            self.hit(
                "structure-extra-extern-block-no-link",
                foreign_module,
                "Extern block has no local link attribute; build script may provide linkage",
            );
        }

        syn::visit::visit_item_foreign_mod(self, foreign_module);
    }

    pub(super) fn inspect_item_mod(&mut self, module: &syn::ItemMod) {
        if let Some((_, items)) = &module.content {
            if items.len() == 1 && matches!(&items[0], syn::Item::Use(_) | syn::Item::Type(_)) {
                self.hit(
                    "structure-single-item-wrapper-module",
                    module,
                    "Module contains a single reexport/type alias",
                );
            }

            if items.len() >= 40 {
                self.hit(
                    "structure-oversized-module",
                    module,
                    "Inline module has at least forty direct items",
                );
            }

            let public = items
                .iter()
                .filter(|index| match index {
                    syn::Item::Fn(function) => is_public(&function.vis),
                    syn::Item::Struct(text) => is_public(&text.vis),
                    syn::Item::Enum(error) => is_public(&error.vis),
                    syn::Item::Type(type_value) => is_public(&type_value.vis),
                    _ => false,
                })
                .count();

            if items.len() >= 5 && public * 4 >= items.len() * 3 {
                self.hit(
                    "structure-broad-public-surface",
                    module,
                    "At least 75% of five or more direct items are public",
                );
            }
        }

        syn::visit::visit_item_mod(self, module);
    }

    pub(super) fn inspect_item_use(&mut self, import: &syn::ItemUse) {
        let text = compact(&import.tree);

        if text.contains('*') {
            self.hit(
                "structure-wildcard-import",
                import,
                "Wildcard use obscures imported names",
            );
        }

        if text.matches("super::").count() >= 3 {
            self.hit(
                "structure-deep-relative-import",
                import,
                "Import traverses at least three parent levels",
            );
        }

        fn aliases(type_value: &syn::UseTree, output: &mut Vec<bool>) {
            match type_value {
                syn::UseTree::Rename(result) => output.push(result.ident == result.rename),
                syn::UseTree::Path(pointer) => aliases(&pointer.tree, output),
                syn::UseTree::Group(g) => {
                    for type_value in &g.items {
                        aliases(type_value, output)
                    }
                }
                _ => {}
            }
        }
        let mut found = Vec::new();
        aliases(&import.tree, &mut found);
        if found.contains(&true) {
            self.hit(
                "structure-extra-import-self-alias",
                import,
                "Import aliases a symbol to its existing name",
            );
        }

        syn::visit::visit_item_use(self, import);
    }

    pub(super) fn inspect_item_impl(&mut self, implementation: &syn::ItemImpl) {
        let methods: Vec<_> = implementation
            .items
            .iter()
            .filter_map(|index| {
                if let syn::ImplItem::Fn(function) = index {
                    Some(function)
                } else {
                    None
                }
            })
            .collect();
        let delegation = regex::Regex::new(r"self\.[A-Za-z_]\w*\.[A-Za-z_]\w*\(").unwrap();
        let delegated = methods
            .iter()
            .filter(|member| {
                member.block.stmts.len() == 1 && delegation.is_match(&compact(&member.block))
            })
            .count();

        if methods.len() >= 3 && delegated == methods.len() {
            self.hit("structure-manual-forwarding-impl",implementation,"Three or more single-statement methods call through self fields; lexical forwarding candidate");
        }

        let roles: std::collections::HashSet<_> = methods
            .iter()
            .flat_map(|member| {
                let result = regex::Regex::new(r"self\.([A-Za-z_]\w*)").unwrap();

                result
                    .captures_iter(&compact(&member.block))
                    .map(|character| character[1].to_owned())
                    .collect::<Vec<_>>()
            })
            .collect();

        if methods.len() >= 3
            && roles.len() >= 3
            && methods
                .iter()
                .all(|member| compact(&member.block).matches("self.").count() == 1)
        {
            self.hit(
                "structure-type-method-role-mix",
                implementation,
                "Methods each touch one of at least three disjoint fields",
            );
        }

        syn::visit::visit_item_impl(self, implementation);
    }
}
