use std::collections::HashMap;
use std::slice;

use functions::*;
use x_ast::{Expression, ExpressionKind, Program, Statement, Type};
use x_diagnostics::CompilationFailure;
use x_diagnostics::contract_messages;

const MAX_SHAREABLE_TYPE_DEPTH: usize = 32;

#[derive(Clone)]
struct FunctionShape {
    pure: bool,
    throws: bool,
    parameters: Vec<Type>,
    result: Type,
}

fn collect_structures(statements: &[Statement], structures: &mut HashMap<String, Vec<Type>>) {
    for statement in statements {
        if let Statement::StructDeclaration(definition) = statement {
            structures.insert(
                definition.name.clone(),
                definition
                    .fields
                    .iter()
                    .map(|(_, value_type)| value_type.clone())
                    .collect(),
            );
        }
    }
}

fn shareable_storage(
    value_type: &Type,
    structures: &HashMap<String, Vec<Type>>,
    depth: usize,
) -> bool {
    if depth > MAX_SHAREABLE_TYPE_DEPTH {
        return false;
    }

    if value_type.is_scalar() {
        return true;
    }

    match value_type {
        _ => false,
    }
}

fn shareable_capture(value_type: &Type, structures: &HashMap<String, Vec<Type>>) -> bool {
    value_type.is_scalar()
        || matches!(value_type, Type::Reference { is_mut: false, inner, .. }
        if shareable_storage(inner, structures, 0))
}

fn collect_functions(statements: &[Statement], functions: &mut HashMap<String, FunctionShape>) {
    for statement in statements {
        match statement {
            Statement::Function {
                name,
                parameters: current_parameters,
                return_type,
                is_pure,
                is_throws,
                ..
            } => {
                functions.insert(
                    name.clone(),
                    FunctionShape {
                        pure: *is_pure,
                        throws: *is_throws,
                        parameters: current_parameters
                            .iter()
                            .map(|pointer| pointer.value_type.clone())
                            .collect(),
                        result: return_type.clone(),
                    },
                );
            }
            Statement::ImplDeclaration(definition) => {
                collect_functions(&definition.methods, functions)
            }
            Statement::TraitDeclaration(definition) => {
                for method in definition.methods.values().flatten() {
                    collect_functions(slice::from_ref(method), functions);
                }
            }
            _ => {}
        }
    }
}

struct ExpressionContracts<'a> {
    functions: &'a HashMap<String, FunctionShape>,
    structures: &'a HashMap<String, Vec<Type>>,
    depth: usize,
    error: Option<CompilationFailure>,
}

mod functions;
