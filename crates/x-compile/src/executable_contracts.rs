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

mod functions;
