mod dependencies;

use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use super::numeric;
use crate::errors::CodeGenerationError;
use x_ast::Operator;
use x_ast::{Expression, ExpressionKind, Type, UnaryOperator};
use x_diagnostics::backend_messages;
use x_mir::Place;

#[derive(Clone, Copy)]
struct InvariantRoot<'a> {
    place: &'a Place,
    receiver: &'a x_ast::InvariantBinding,
}

fn invariant_place(expression: &Expression, root: InvariantRoot<'_>) -> Option<Place> {
    match &expression.kind {
        _ if root.receiver.refers_to(expression) => Some(root.place.clone()),
        ExpressionKind::FieldAccess { object, field } => Some(Place::Field {
            base: Box::new(invariant_place(object, root)?),
            field_name: field.clone(),
            value_type: expression.value_type.clone(),
        }),
        _ => None,
    }
}

fn unsupported(expression: &Expression) -> CodeGenerationError {
    CodeGenerationError::InvalidMir(
        backend_messages::struct_invariant_contains_an_unsupported_executable_expression_value(
            &expression.kind,
        ),
    )
}
