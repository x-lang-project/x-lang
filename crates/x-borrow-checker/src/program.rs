use std::collections::HashMap;
use std::sync::Arc;

use super::return_summaries;
use x_ast::{
    DeclarationKey, ExpressionKind, Pattern, Program, SourceError, SourceSpan, Statement, Type,
};

pub(super) fn nominal_owner_name(value_type: &Type) -> String {
    match value_type {
        Type::Custom(name) | Type::GenericInstance { name, .. } => name.clone(),
        _ => value_type.to_string(),
    }
}

fn method_receivers(program: &Program) -> HashMap<(String, String), (String, Type)> {
    let mut methods = HashMap::new();

    for statement in &program.statements {
        match statement {
            Statement::ImplDeclaration(definition) => {
                let owner = nominal_owner_name(&definition.type_name);

                for statement in &definition.methods {
                    if let Statement::Function {
                        name, parameters, ..
                    } = statement
                        && let Some(receiver) = parameters
                            .first()
                            .filter(|parameter| parameter.is_receiver())
                    {
                        methods.insert(
                            (owner.clone(), name.clone()),
                            (format!("{owner}::{name}"), receiver.value_type.clone()),
                        );
                    }
                }
            }
            Statement::Function {
                name, parameters, ..
            } => {
                if let Some(owner) = x_ast::intrinsics::function_owner_name(name)
                    && let Some(receiver) = parameters
                        .first()
                        .filter(|parameter| parameter.is_receiver())
                {
                    methods.insert(
                        (
                            owner.into(),
                            x_ast::intrinsics::function_leaf_name(name).into(),
                        ),
                        (name.clone(), receiver.value_type.clone()),
                    );
                }
            }
            _ => {}
        }
    }

    methods
}
