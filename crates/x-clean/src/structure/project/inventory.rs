use super::*;

pub(super) struct Inventory<'a> {
    pub(super) source: &'a Source,
    pub(super) index: usize,
    pub(super) module: String,
    pub(super) rows: Vec<Record>,
}
impl Inventory<'_> {
    fn add(&mut self, kind: RecordKind, name: String, shape: String, count: &impl Spanned) {
        self.rows.push(Record {
            source: self.index,
            offset: self.source.offset(count.span()),
            kind,
            name,
            shape,
            module: self.module.clone(),
        });
    }

    fn policies(
        &mut self,
        attributes: &[syn::Attribute],
        policy: AttributePolicy,
        name: String,
        count: &impl Spanned,
    ) {
        let policy_kind = policy.kind();
        let policy = attributes
            .iter()
            .filter(|first_value| first_value.path().is_ident(policy.syntax()))
            .map(compact)
            .collect::<Vec<_>>()
            .join(";");
        self.add(policy_kind, name, policy, count);
    }
}

impl<'ast> Visit<'ast> for Inventory<'_> {
    fn visit_item_mod(&mut self, current_module: &'ast syn::ItemMod) {
        let previous = self.module.clone();
        self.module = format!("{previous}::{}", current_module.ident);
        self.add(
            RecordKind::Module,
            current_module.ident.to_string(),
            compact(current_module),
            current_module,
        );
        self.policies(
            &current_module.attrs,
            AttributePolicy::Cfg,
            current_module.ident.to_string(),
            current_module,
        );
        syn::visit::visit_item_mod(self, current_module);
        self.module = previous;
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        self.policies(
            &function.attrs,
            AttributePolicy::Cfg,
            function.sig.ident.to_string(),
            function,
        );
        syn::visit::visit_item_fn(self, function);
    }

    fn visit_item_struct(&mut self, structure: &'ast syn::ItemStruct) {
        self.add(
            RecordKind::Record,
            structure.ident.to_string(),
            compact(&structure.fields),
            structure,
        );
        self.policies(
            &structure.attrs,
            AttributePolicy::Derive,
            compact(&structure.fields),
            structure,
        );
        syn::visit::visit_item_struct(self, structure);
    }

    fn visit_item_enum(&mut self, enumeration: &'ast syn::ItemEnum) {
        let shape = enumeration
            .variants
            .iter()
            .map(compact)
            .collect::<Vec<_>>()
            .join(";");
        self.add(
            RecordKind::Enum,
            enumeration.ident.to_string(),
            shape.clone(),
            enumeration,
        );
        self.policies(
            &enumeration.attrs,
            AttributePolicy::Derive,
            shape,
            enumeration,
        );
        syn::visit::visit_item_enum(self, enumeration);
    }

    fn visit_item_macro(&mut self, macro_item: &'ast syn::ItemMacro) {
        if macro_item.mac.path.is_ident("macro_rules") {
            self.add(
                RecordKind::Macro,
                macro_item
                    .ident
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                compact(&macro_item.mac.tokens),
                macro_item,
            );
        }

        syn::visit::visit_item_macro(self, macro_item);
    }

    fn visit_item_impl(&mut self, implementation: &'ast syn::ItemImpl) {
        if implementation.trait_.is_none() {
            self.add(
                RecordKind::Impl,
                compact(&implementation.self_ty),
                String::new(),
                implementation,
            );
        }

        syn::visit::visit_item_impl(self, implementation);
    }

    fn visit_expr_match(&mut self, count: &'ast syn::ExprMatch) {
        let shape = count
            .arms
            .iter()
            .map(|first_value| compact(&first_value.body))
            .collect::<Vec<_>>()
            .join(";");

        if count.arms.len() >= 2
            && count.arms.iter().all(|first_value| {
                compact(&first_value.pat).contains("::")
                    && compact(&first_value.body).contains("::")
            })
        {
            self.add(RecordKind::ErrorMap, String::new(), shape, count);
        }

        syn::visit::visit_expr_match(self, count);
    }

    fn visit_item_use(&mut self, import: &'ast syn::ItemUse) {
        self.add(
            RecordKind::Use,
            String::new(),
            compact(&import.tree),
            import,
        );
        if is_public(&import.vis) {
            self.add(
                RecordKind::Reexport,
                String::new(),
                compact(&import.tree),
                import,
            );
        }

        fn aliases(type_value: &syn::UseTree, prefix: String, output: &mut Vec<(String, String)>) {
            match type_value {
                syn::UseTree::Rename(result) => output.push((
                    result.rename.to_string(),
                    format!("{prefix}{}", result.ident),
                )),
                syn::UseTree::Path(pointer) => aliases(
                    &pointer.tree,
                    format!("{prefix}{}::", pointer.ident),
                    output,
                ),
                syn::UseTree::Group(g) => {
                    for type_value in &g.items {
                        aliases(type_value, prefix.clone(), output)
                    }
                }
                _ => {}
            }
        }
        let mut pairs = Vec::new();
        aliases(&import.tree, String::new(), &mut pairs);
        for (name, shape) in pairs {
            self.add(RecordKind::ImportAlias, name, shape, import);
        }

        syn::visit::visit_item_use(self, import);
    }
}
