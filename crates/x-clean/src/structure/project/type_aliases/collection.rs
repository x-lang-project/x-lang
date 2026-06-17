use syn::{spanned::Spanned, visit::Visit};

use super::index::{Alias, AliasIndex};

pub(super) struct Collector<'a> {
    pub(super) index: &'a mut AliasIndex,
    pub(super) source: usize,
    pub(super) data: &'a crate::Source,
    pub(super) scope: usize,
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let previous = self.scope;
        self.index.unresolved(previous, item.ident.to_string());
        self.scope = self
            .index
            .child_module_scope(previous, &item.ident.to_string());
        syn::visit::visit_item_mod(self, item);
        self.scope = previous;
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let previous = self.scope;
        self.scope = self.index.block_scope(previous);
        syn::visit::visit_block(self, block);
        self.scope = previous;
    }

    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        let alias = Alias::new(self.source, self.scope, item, self.data.offset(item.span()));
        self.index.add(alias);
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.index.unresolved(self.scope, item.ident.to_string());
        syn::visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        self.index.unresolved(self.scope, item.ident.to_string());
        syn::visit::visit_item_enum(self, item);
    }

    fn visit_item_union(&mut self, item: &'ast syn::ItemUnion) {
        self.index.unresolved(self.scope, item.ident.to_string());
        syn::visit::visit_item_union(self, item);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        self.index.unresolved(self.scope, item.ident.to_string());
        syn::visit::visit_item_trait(self, item);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.imports(&item.tree, None);
    }
}

impl Collector<'_> {
    fn imports(&mut self, tree: &syn::UseTree, parent: Option<&str>) {
        match tree {
            syn::UseTree::Path(path) => self.imports(&path.tree, Some(&path.ident.to_string())),
            syn::UseTree::Name(name) => {
                let name = name.ident.to_string();
                let binding = if name == "self" {
                    parent.unwrap_or(&name)
                } else {
                    &name
                };

                self.index.unresolved(self.scope, binding.to_owned());
            }
            syn::UseTree::Rename(rename) if rename.rename != "_" => {
                self.index.unresolved(self.scope, rename.rename.to_string());
            }
            syn::UseTree::Rename(_) => {}
            syn::UseTree::Glob(_) => self.index.unresolved_glob(self.scope),
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.imports(tree, parent);
                }
            }
        }
    }
}
