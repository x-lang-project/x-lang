use std::collections::HashSet;

use crate::{TypeChecker, errors::TypeError};
use x_ast::{DeclarationKey, Pattern, Statement, Type};
use x_diagnostics::frontend_messages::FrontendIssue;

mod purity;

impl TypeChecker {
    pub(super) fn check_source_statements(
        &mut self,
        statements: &mut [Statement],
        source_names: &HashSet<String>,
    ) -> Result<(), TypeError> {
        for statement in statements {
            // Only bodyless catalogue intrinsics skip source-body checking.
            if matches!(statement, Statement::Function { body: None, .. })
                && namespaced_statement_name(statement)
                    .is_some_and(|name| !source_names.contains(name))
            {
                continue;
            }

            purity::validate_pure_bodies(statement)?;
            self.resolve_overload_declaration(statement)?;
            self.check_statement(statement)?;
        }

        Ok(())
    }

    fn resolve_overload_declaration(&mut self, statement: &mut Statement) -> Result<(), TypeError> {
        if let Statement::Function {
            name,
            parameters,
            is_multi: true,
            ..
        } = statement
        {
            let parameter_types = parameters
                .iter()
                .map(|parameter| parameter.value_type.clone())
                .collect::<Vec<_>>();
            let original_key = DeclarationKey::Function(name.clone(), parameter_types.clone());
            let chosen = self
                .symbols
                .multi_functions
                .get(name)
                .and_then(|candidates| {
                    candidates
                        .iter()
                        .find(|candidate| candidate.signature.parameter_types == parameter_types)
                })
                .ok_or_else(|| {
                    TypeError::Invariant(FrontendIssue::MissingIndexedOverload {
                        name: name.clone(),
                    })
                })?;
            *name = chosen.mangled_name.clone();

            if let Some(origin) = self.declaration_sources.get(&original_key).copied() {
                self.declaration_sources.insert(
                    DeclarationKey::Function(name.clone(), parameter_types),
                    origin,
                );
            }
        }

        Ok(())
    }
}

pub(super) fn collect_source_namespace_names(
    statements: &[Statement],
    names: &mut HashSet<String>,
) {
    for statement in statements {
        if let Statement::Block { statements } = statement {
            collect_source_namespace_names(statements, names);
        } else if let Some(name) = namespaced_statement_name(statement) {
            names.insert(name.to_owned());
        }
    }
}

pub(super) fn namespaced_statement_name(statement: &Statement) -> Option<&str> {
    match statement {
        Statement::Function { name, .. }
        | Statement::TypeAlias { name, .. }
        | Statement::Constant { name, .. }
        | Statement::VariableDeclaration {
            pattern: Pattern::Identifier(name),
            ..
        } => name.contains("::").then_some(name),
        Statement::ImplDeclaration(impl_definition) => match &impl_definition.type_name {
            Type::Custom(name) | Type::GenericInstance { name, .. } => {
                name.contains("::").then_some(name)
            }
            _ => None,
        },
        _ => None,
    }
}
