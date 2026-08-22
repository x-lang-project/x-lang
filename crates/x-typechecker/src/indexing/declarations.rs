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

    pub(super) fn index_trait_declaration(
        &mut self,
        trait_definition: &x_ast::TraitDefinition,
    ) -> Result<(), TypeError> {
        let trait_name = self.qualify_indexed_name(&trait_definition.name);
        let mut methods = HashMap::new();

        for (method_name, method_statement) in &trait_definition.methods {
            let Some(Statement::Function {
                parameters,
                return_type,
                is_pure,
                is_throws,
                is_static,
                is_multi,
                ..
            }) = method_statement
            else {
                continue;
            };

            if *is_multi {
                return Err(TypeError::invalid(
                    FrontendMessage::UnsupportedMultiDeclaration {
                        name: format!("{trait_name}::{method_name}"),
                    },
                ));
            }

            methods.insert(
                method_name.clone(),
                crate::typed_function_signature!(
                    parameters,
                    return_type,
                    *is_pure,
                    *is_throws,
                    *is_static,
                ),
            );
        }

        self.symbols.traits.insert(
            trait_name.clone(),
            TraitInfo {
                name: trait_name.clone(),
                methods,
            },
        );
        let mut qualified_trait = trait_definition.clone();
        qualified_trait.name = trait_name.clone();
        self.symbols
            .trait_ast_definitions
            .insert(trait_name, qualified_trait);

        Ok(())
    }
}
