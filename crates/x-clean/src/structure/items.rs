use super::*;

mod api;
mod boundaries;
mod types;
pub(super) struct Inspector<'a> {
    pub source: &'a Source,
    pub findings: &'a mut Vec<Finding>,
}
impl Inspector<'_> {
    fn hit(&mut self, identifier: &str, node: &impl Spanned, message: impl Into<String>) {
        crate::ReviewSink {
            source: self.source,
            findings: self.findings,
        }
        .at_node(identifier, node, message);
    }
}

impl<'ast> Visit<'ast> for Inspector<'_> {
    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        self.inspect_attribute(attribute);
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        self.inspect_item_function(function);
    }

    fn visit_impl_item_fn(&mut self, count: &'ast syn::ImplItemFn) {
        self.inspect_impl_item_function(count);
    }

    fn visit_item_struct(&mut self, structure: &'ast syn::ItemStruct) {
        self.inspect_item_struct(structure);
    }

    fn visit_item_enum(&mut self, enumeration: &'ast syn::ItemEnum) {
        self.inspect_item_enum(enumeration);
    }

    fn visit_item_trait(&mut self, trait_definition: &'ast syn::ItemTrait) {
        self.inspect_item_trait(trait_definition);
    }

    fn visit_item_static(&mut self, static_item: &'ast syn::ItemStatic) {
        self.inspect_item_static(static_item);
    }

    fn visit_item_foreign_mod(&mut self, foreign_module: &'ast syn::ItemForeignMod) {
        self.inspect_item_foreign_mod(foreign_module);
    }

    fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
        self.inspect_item_mod(module);
    }

    fn visit_item_use(&mut self, import: &'ast syn::ItemUse) {
        self.inspect_item_use(import);
    }

    fn visit_item_impl(&mut self, implementation: &'ast syn::ItemImpl) {
        self.inspect_item_impl(implementation);
    }
}
