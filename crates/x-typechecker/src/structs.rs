mod invariants;
mod storage;

use std::collections::HashMap;

use crate::errors;
use crate::expression;
use crate::tail_helpers::qualified_path_suffix_matches;
use crate::{TypeChecker, symbols::mangle_type_components};
use x_ast::DeclarationKey;
use x_ast::{Statement, StructDefinition, Type};
use x_diagnostics::frontend_messages::FrontendIssue;

impl TypeChecker {
    pub(crate) fn resolve_generic_struct_key(&self, name: &str) -> Option<String> {
        if let Some(alias) = self.symbols.imported_aliases.get(name)
            && self.symbols.records.templates().contains_key(alias)
        {
            return Some(alias.clone());
        }

        if self.symbols.records.templates().contains_key(name) {
            return Some(name.to_string());
        }

        let mut matches = self
            .symbols
            .records
            .templates()
            .keys()
            .filter(|candidate| qualified_name_matches(candidate, name));

        let first = matches.next()?.clone();

        if matches.next().is_none() {
            Some(first)
        } else {
            None
        }
    }

    pub fn instantiate_struct(
        &mut self,
        name: &str,
        type_arguments: &[Type],
    ) -> Result<String, errors::TypeError> {
        self.with_type_resolution(
            crate::errors::TypeResource::GenericStructInstantiationDepth,
            |checker| checker.instantiate_struct_inner(name, type_arguments),
        )
    }

    fn instantiate_struct_inner(
        &mut self,
        name: &str,
        type_arguments: &[Type],
    ) -> Result<String, errors::TypeError> {
        let key = (name.to_string(), type_arguments.to_vec());

        if let Some(existing) = self.symbols.records.instantiations().get(&key) {
            return Ok(existing.clone());
        }

        let generic_struct = self
            .symbols
            .records
            .templates()
            .get(name)
            .cloned()
            .ok_or_else(|| {
                errors::TypeError::GenericIssue(FrontendIssue::UnknownGenericStruct {
                    name: name.to_owned(),
                })
            })?;

        let generic_parameters = generic_struct.generic_parameters.ok_or_else(|| {
            errors::TypeError::Invariant(FrontendIssue::MissingGenericParameters {
                name: name.to_owned(),
            })
        })?;

        let type_parameters = generic_parameters
            .iter()
            .filter_map(x_ast::GenericParameter::non_lifetime_name)
            .cloned()
            .collect::<Vec<_>>();

        if type_arguments.len() != type_parameters.len() {
            return Err(errors::TypeError::GenericIssue(
                FrontendIssue::GenericStructArity {
                    name: name.to_owned(),
                    expected: type_parameters.len(),
                    got: type_arguments.len(),
                },
            ));
        }

        let mut type_map = HashMap::new();

        for (parameter_name, concrete) in type_parameters.into_iter().zip(type_arguments) {
            type_map.insert(parameter_name, concrete.clone());
        }

        let suffix = mangle_type_components(type_arguments);
        let mangled_name = self
            .symbols
            .vacant_generated_name(&format!("{name}_{suffix}"))
            .map_err(errors::TypeError::Invariant)?;

        let reservation = self.symbols.records.reserve_instantiation(
            key,
            mangled_name.clone(),
            generic_struct.layout,
        );

        let specialised_fields = generic_struct
            .fields
            .into_iter()
            .map(|(field_name, field_type)| {
                (field_name, self.substitute_type(&field_type, &type_map))
            })
            .collect::<Vec<_>>();

        self.symbols
            .records
            .complete_instantiation(reservation, specialised_fields.clone());

        let mut invariant = generic_struct.invariant;

        if let Some(invariant) = &mut invariant {
            expression::substitute_types_in_expression(&mut invariant.predicate, self, &type_map);
            self.register_struct_invariant(mangled_name.clone(), invariant);
        }

        let specialised_declaration = Statement::StructDeclaration(StructDefinition {
            name: mangled_name.clone(),
            generic_parameters: None,
            fields: specialised_fields,
            invariant,
            layout: generic_struct.layout,
            derives: generic_struct.derives,
        });

        if !self.program.statements.iter().any(
            |statement| matches!(statement, Statement::StructDeclaration(existing) if existing.name == mangled_name),
        ) {
            self.program.statements.push(specialised_declaration.clone());
        }

        if !self.pending_monomorphs.iter().any(
            |statement| matches!(statement, Statement::StructDeclaration(existing) if existing.name == mangled_name),
        ) {
            if let Some(origin) = self
                .declaration_sources
                .get(&DeclarationKey::Struct(name.to_string()))
                .copied()
            {
                self.declaration_sources
                    .insert(DeclarationKey::Struct(mangled_name.clone()), origin);
            }

            self.pending_monomorphs.push(specialised_declaration);
        }

        Ok(mangled_name)
    }
}

fn qualified_name_matches(registered: &str, requested: &str) -> bool {
    if registered == requested {
        return true;
    }

    qualified_path_suffix_matches(registered, requested)
}
