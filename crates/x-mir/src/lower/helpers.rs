use crate::errors;
use x_ast::Statement;
use x_ast::StructDefinition;
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::backend_messages;

pub(crate) fn constant_branch_value<'a>(
    condition: &Expression,
    then_block: &'a [Statement],
    else_block: Option<&'a [Statement]>,
) -> Option<&'a Expression> {
    let selected = match condition.kind {
        ExpressionKind::Boolean(true) => then_block,
        ExpressionKind::Boolean(false) => else_block?,
        _ => return None,
    };

    let mut statements = selected
        .iter()
        .filter(|statement| !matches!(statement, Statement::Comment(_)));

    match (statements.next(), statements.next()) {
        (Some(Statement::Expression { expression }), None) => Some(expression),
        _ => None,
    }
}

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

pub(crate) fn array_repeat_length(expression: &Expression) -> Result<usize, errors::MirLowerError> {
    let ExpressionKind::Integer(raw_count) = expression.kind else {
        return Err(errors::MirLowerError::UnsupportedExpression(backend_messages::array_repetition_length_must_remain_a_compile_time_integer_through_mir_lowering()));
    };

    usize::try_from(raw_count).map_err(|_| {
        errors::MirLowerError::UnsupportedExpression(
            backend_messages::array_repetition_length_must_be_non_negative_and_fit_usize(),
        )
    })
}

pub(crate) fn resolved_expression_type(value: &Expression) -> Option<Type> {
    (!matches!(value.value_type, Type::Unknown | Type::Infer(_))).then(|| value.value_type.clone())
}

pub(crate) fn resolved_tuple_type(value: &Expression, arity: usize) -> Option<Type> {
    match &value.value_type {
        Type::Tuple(items) if items.len() == arity => Some(value.value_type.clone()),
        _ => None,
    }
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

pub(crate) fn canonical_array_expression_type(value_type: &Type, length: usize) -> Type {
    match value_type {
        Type::Array(inner) => Type::FixedSizedArray(inner.clone(), length),
        _ => value_type.clone(),
    }
}
