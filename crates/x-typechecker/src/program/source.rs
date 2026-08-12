use std::collections::HashSet;

use crate::TypeChecker;
use x_ast::{DeclarationKey, Pattern, Statement, Type};
use x_diagnostics::frontend_messages::FrontendIssue;

mod purity;

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
