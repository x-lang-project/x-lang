use crate::errors;
use x_ast::Statement;
use x_ast::StructDefinition;
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::backend_messages;

pub(crate) fn literal_string_value(literal: &x_ast::StringLiteral) -> Option<String> {
    let mut value = String::new();

    for part in &literal.parts {
        match part {
            x_ast::StringPart::Text(text) => value.push_str(text),
            x_ast::StringPart::Interpolation(_) => return None,
        }
    }

    Some(value)
}

pub(crate) use crate::type_queries::operand_type;

pub(crate) fn is_codegen_concrete_struct(definition: &StructDefinition) -> bool {
    let has_type_generic = definition
        .generic_parameters
        .as_ref()
        .is_some_and(|parameters| {
            parameters
                .iter()
                .any(|parameter| !matches!(parameter, x_ast::GenericParameter::Lifetime { .. }))
        });

    !has_type_generic
        && definition
            .fields
            .iter()
            .all(|(_, value_type)| !type_contains_type_parameter(value_type))
}

pub(crate) fn type_contains_type_parameter(value_type: &Type) -> bool {
    value_type.any_nested_type(|inner| matches!(inner, Type::TypeParameter(_)))
}
