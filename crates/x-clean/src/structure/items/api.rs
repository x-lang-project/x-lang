use super::*;

impl Inspector<'_> {
    pub(super) fn signature(
        &mut self,
        current_signature: &syn::Signature,
        visibility: &syn::Visibility,
    ) {
        if is_public(visibility) {
            if current_signature
                .inputs
                .iter()
                .any(|first_value| matches!(first_value,syn::FnArg::Typed(pointer) if compact(&pointer.ty)=="bool"))
            {
                self.hit(
                    "structure-boolean-mode-api",
                    current_signature,
                    "Public API accepts a bool mode argument",
                );
            }

            let shape = compact(current_signature);

            if shape.matches('<').count() >= 4 || shape.matches(',').count() >= 8 {
                self.hit(
                    "structure-complex-public-signature",
                    current_signature,
                    "Public signature has >=4 generic delimiters or >=8 commas",
                );
            }

            if shape.contains("*const") || shape.contains("*mut") {
                self.hit(
                    "structure-extra-raw-pointer-public-api",
                    current_signature,
                    "Public signature exposes raw pointers; review contract",
                );
            }

            if current_signature.abi.as_ref().is_some_and(|first_value| {
                first_value
                    .name
                    .as_ref()
                    .is_some_and(|current_count| current_count.value() == "Rust")
            }) {
                self.hit(
                    "structure-extra-public-extern-rust-abi",
                    current_signature,
                    "Public extern function uses Rust ABI",
                );
            }
        }

        if current_signature.generics.type_params().count() >= 5 {
            self.hit(
                "structure-extra-generic-parameter-count",
                current_signature,
                "Signature has at least five type parameters",
            );
        }

        if current_signature.generics.lifetimes().count() >= 4 {
            self.hit(
                "structure-extra-lifetime-parameter-count",
                current_signature,
                "Signature has at least four lifetime parameters",
            );
        }
    }

    pub(super) fn attributes(&mut self, current_attributes: &[syn::Attribute]) {
        for first_value in current_attributes {
            let text = compact(first_value);

            if first_value.path().is_ident("allow") || first_value.path().is_ident("expect") {
                let offset = self.source.offset(first_value.span());
                let start = self
                    .source
                    .line_offset(first_value.span().start().line.saturating_sub(2).max(1));
                let nearby =
                    &self.source.text[start.min(offset)..offset.min(self.source.text.len())];

                if !text.contains("reason=") && !nearby.contains("//") {
                    self.hit(
                        "structure-lint-suppression-no-rationale",
                        first_value,
                        "Lint suppression has no reason metadata or nearby comment",
                    );
                }

                if matches!(first_value.style, syn::AttrStyle::Inner(_)) {
                    self.hit(
                        "structure-lint-suppression-scope",
                        first_value,
                        "Inner suppression covers the containing module/crate",
                    );
                }
            }

            if first_value.path().is_ident("deprecated") {
                if !text.contains("note=") {
                    self.hit(
                        "structure-missing-deprecation-migration",
                        first_value,
                        "Deprecation has no note describing migration",
                    );
                }

                if !text.contains("since=") {
                    self.hit(
                        "structure-extra-deprecated-no-since",
                        first_value,
                        "Deprecation omits version metadata",
                    );
                }
            }

            if text.contains("repr(packed") {
                self.hit(
                    "structure-extra-repr-packed",
                    first_value,
                    "Packed layout requires alignment-sensitive access review",
                );
            }

            for (needle, identifier, message) in [
                (
                    "cfg(any())",
                    "structure-extra-cfg-any-empty",
                    "Empty any cfg predicate is always false",
                ),
                (
                    "cfg(all())",
                    "structure-extra-cfg-all-empty",
                    "Empty all cfg predicate is always true",
                ),
                (
                    "cfg(not(feature=",
                    "structure-extra-cfg-feature-negation",
                    "Negated feature gate can complicate additive feature combinations",
                ),
            ] {
                if text.contains(needle) {
                    self.hit(identifier, first_value, message);
                }
            }

            if first_value.path().is_ident("must_use")
                && matches!(first_value.meta, syn::Meta::Path(_))
            {
                self.hit(
                    "structure-extra-must-use-no-message",
                    first_value,
                    "must_use has no actionable diagnostic message",
                );
            }

            let paths = first_value
                .path()
                .is_ident("derive")
                .then(|| {
                    first_value.parse_args_with(
                        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                    )
                })
                .and_then(Result::ok);

            if let Some(paths) = paths {
                let mut names = std::collections::HashSet::new();

                for pointer in paths {
                    if !names.insert(compact(&pointer)) {
                        self.hit(
                            "structure-extra-duplicate-derive",
                            first_value,
                            "Same trait appears twice in derive list",
                        );
                    }
                }
            }
        }
    }
}
