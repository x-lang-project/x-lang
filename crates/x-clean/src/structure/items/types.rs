use super::*;

impl Inspector<'_> {
    pub(super) fn inspect_item_struct(&mut self, structure: &syn::ItemStruct) {
        let fields: Vec<_> = structure.fields.iter().collect();
        let count_type = |needle: &str| {
            fields
                .iter()
                .filter(|field| compact(&field.ty).contains(needle))
                .count()
        };

        if count_type("Vec<") + count_type("Map<") >= 2 {
            self.hit(
                "structure-parallel-collection-fields",
                structure,
                "Record contains multiple collections; indexing synchronisation requires review",
            );
        }

        if fields.len() >= 4
            && fields
                .iter()
                .filter(|field| {
                    ["u8", "u16", "u32", "u64", "usize", "i32", "bool", "String"]
                        .contains(&compact(&field.ty).as_str())
                })
                .count()
                >= 4
        {
            self.hit(
                "structure-primitive-domain-field-clusters",
                structure,
                "Record contains at least four primitive/String fields",
            );
        }

        if count_type("bool") >= 3 {
            self.hit(
                "structure-extra-bool-field-cluster",
                structure,
                "Record contains at least three boolean fields",
            );
        }

        if count_type("Option<") >= 4 {
            self.hit(
                "structure-extra-option-field-cluster",
                structure,
                "Record contains at least four optional fields",
            );
        }

        if matches!(structure.fields, syn::Fields::Unit) {
            self.hit(
                "structure-extra-unit-struct",
                structure,
                "Unit struct may be intentional marker; review role",
            );
        }

        if matches!(structure.fields, syn::Fields::Unnamed(_)) && fields.len() >= 5 {
            self.hit(
                "structure-extra-tuple-record-arity",
                structure,
                "Tuple record has at least five positional fields",
            );
        }

        for field in &fields {
            if is_public(&field.vis)
                && ["Cell<", "RefCell<", "Mutex<", "RwLock<"]
                    .iter()
                    .any(|text| compact(&field.ty).contains(text))
            {
                self.hit(
                    "structure-extra-public-mutable-field",
                    field,
                    "Public field exposes an interior mutable container",
                );
            }
        }

        let public = fields.iter().filter(|field| is_public(&field.vis)).count();

        if fields.len() >= 3 && public > 0 && public < fields.len() {
            self.hit(
                "structure-visibility-outlier",
                structure,
                "Sibling record fields mix public and private visibility",
            );
        }

        syn::visit::visit_item_struct(self, structure);
    }

    pub(super) fn inspect_item_enum(&mut self, enumeration: &syn::ItemEnum) {
        if enumeration.variants.is_empty() {
            self.hit(
                "structure-extra-empty-enum",
                enumeration,
                "Empty enum is uninhabited; review representation intent",
            );
        }

        if enumeration.variants.len() >= 16 {
            self.hit(
                "structure-extra-huge-enum",
                enumeration,
                "Enum has at least sixteen variants",
            );
        }

        let unit = enumeration
            .variants
            .iter()
            .filter(|value| matches!(value.fields, syn::Fields::Unit))
            .count();

        if unit > 0 && unit < enumeration.variants.len() {
            self.hit(
                "structure-extra-mixed-enum-payload",
                enumeration,
                "Enum mixes unit and payload variants",
            );
        }

        let mut values = Vec::new();

        for value in &enumeration.variants {
            if let Some((
                _,
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(left_value),
                    ..
                }),
            )) = &value.discriminant
            {
                values.extend(left_value.base10_parse::<i64>());
            }
        }

        if values.windows(2).any(|value| value[1] > value[0] + 1) {
            self.hit(
                "structure-extra-enum-discriminant-gap",
                enumeration,
                "Explicit discriminants contain a positive gap",
            );
        }

        let names: Vec<_> = enumeration
            .variants
            .iter()
            .map(|value| value.ident.to_string())
            .collect();

        if names.len() >= 3
            && names
                .iter()
                .all(|value| value.starts_with(&enumeration.ident.to_string()))
        {
            self.hit(
                "structure-extra-enum-variant-prefix",
                enumeration,
                "Every variant repeats the enum type name prefix",
            );
        }

        syn::visit::visit_item_enum(self, enumeration);
    }

    pub(super) fn inspect_item_trait(&mut self, trait_definition: &syn::ItemTrait) {
        let methods: Vec<_> = trait_definition
            .items
            .iter()
            .filter_map(|index| {
                if let syn::TraitItem::Fn(function) = index {
                    Some(function)
                } else {
                    None
                }
            })
            .collect();

        if methods.len() >= 12 {
            self.hit(
                "structure-extra-trait-method-count",
                trait_definition,
                "Trait has at least twelve methods",
            );
        }

        if methods
            .iter()
            .filter(|member| member.default.is_some())
            .count()
            >= 8
        {
            self.hit(
                "structure-extra-trait-default-body-count",
                trait_definition,
                "Trait has at least eight default method bodies",
            );
        }

        syn::visit::visit_item_trait(self, trait_definition);
    }
}
