mod dispatch;

mod overloads;

mod contracts;
use std::collections::HashMap;

use crate::FunctionSignature;
use crate::{
    FunctionDefinition, MultiFunctionCandidate, TraitInfo, TypeAliasDefinition, TypeChecker,
    errors::TypeError,
};
use contracts::*;
use x_ast::SourceError;
use x_ast::{Pattern, Statement, Type};
use x_diagnostics::frontend_messages::FrontendMessage;

mod declarations;

struct ExternDeclaration<'a> {
    name: &'a str,
    parameters: &'a [x_ast::ExternParameter],
    return_type: Option<&'a str>,
    abi: Option<&'a str>,
    is_pure: bool,
}

impl TypeChecker {
    fn qualify_indexed_name(&self, name: &str) -> String {
        match &self.current_namespace {
            Some(namespace) if !name.contains("::") => format!("{namespace}::{name}"),
            _ => name.to_string(),
        }
    }

    pub fn index_global_symbols(&mut self, statements: &[Statement]) -> Result<(), TypeError> {
        self.symbols
            .reserved_source_names
            .extend(statements.iter().filter_map(|statement| match statement {
                Statement::Function { name, .. }
                | Statement::ExternFunctionDeclaration { name, .. }
                | Statement::Constant { name, .. }
                | Statement::VariableDeclaration {
                    pattern: Pattern::Identifier(name),
                    ..
                } => Some(name.clone()),
                Statement::StructDeclaration(definition) => Some(definition.name.clone()),
                _ => None,
            }));
        for current_statement in statements {
            self.index_global_statement(current_statement)
                .map_err(|error| {
                    error.at_source(
                        current_statement
                            .declaration_key()
                            .and_then(|key| self.declaration_sources.get(&key).copied()),
                    )
                })?;
        }

        // Validate contracts after indexing later declarations and generated fields.
        let mut signatures = self
            .symbols
            .functions
            .iter()
            .filter(|(_, signature)| signature.is_pure)
            .map(|(name, signature)| (name.clone(), signature.parameter_types.clone()))
            .collect::<Vec<_>>();
        signatures.extend(
            self.symbols
                .generic_functions
                .iter()
                .filter(|(_, definition)| definition.is_pure)
                .map(|(name, definition)| {
                    (
                        name.clone(),
                        definition
                            .parameters
                            .iter()
                            .map(|parameter| parameter.value_type.clone())
                            .collect::<Vec<_>>(),
                    )
                }),
        );
        let methods = self
            .symbols
            .struct_methods
            .iter()
            .flat_map(|(owner, methods)| {
                methods
                    .iter()
                    .filter(|(_, signature)| signature.is_pure)
                    .map(move |(_, signature)| (owner.clone(), signature.parameter_types.clone()))
            })
            .collect::<Vec<_>>();

        for (owner, parameters) in methods {
            let substitutions = HashMap::from([("Self".into(), Type::Custom(owner.clone()))]);
            let concrete = parameters
                .iter()
                .map(|value_type| self.substitute_type(value_type, &substitutions))
                .collect();
            signatures.push((owner, concrete));
        }

        for (name, parameters) in signatures {
            for (index, value_type) in parameters.iter().enumerate() {
                if self.type_invalidates_pure_memory_contract(value_type) {
                    return Err(TypeError::invalid(FrontendMessage::MutablePureParameter {
                        fully_qualified: name.clone(),
                        name: format!("argument {}", index + 1),
                    }));
                }
            }
        }

        Ok(())
    }
}
