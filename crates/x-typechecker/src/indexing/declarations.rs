use super::*;

impl TypeChecker {
    pub(super) fn index_struct_declaration(&mut self, sdef: &x_ast::StructDefinition) {
        let mut sdef = sdef.clone();
        sdef.name = self.qualify_indexed_name(&sdef.name);
        if let Some(invariant) = &sdef.invariant {
            self.register_struct_invariant(sdef.name.clone(), invariant);
        }

        self.symbols.records.index_declaration(sdef);
    }
}
