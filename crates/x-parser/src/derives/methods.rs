use super::*;

pub(super) fn default_expression_for_type(value_type: &Type) -> Expression {
    match value_type {
        Type::Boolean => Expression::new(Type::Boolean, ExpressionKind::Boolean(false)),
        Type::String => string_expression(""),
        Type::Character => Expression::new(Type::Character, ExpressionKind::Character('\0')),
        Type::F32 | Type::F64 => Expression::new(value_type.clone(), ExpressionKind::Float(0.0)),
        Type::I8
        | Type::U8
        | Type::I16
        | Type::U16
        | Type::I32
        | Type::U32
        | Type::I64
        | Type::U64
        | Type::ISize
        | Type::USize => Expression::new(value_type.clone(), ExpressionKind::Integer(0)),
        _ => Expression::new(
            Type::Unknown,
            ExpressionKind::FunctionCall {
                target: x_ast::CallTarget::plain("Default::default"),
                arguments: Vec::new(),
            },
        ),
    }
}

pub(super) fn self_reference_parameter(is_mut: bool) -> Parameter {
    Parameter {
        role: x_ast::ParameterRole::Receiver,
        name: "self".to_string(),
        value_type: Type::Reference {
            lifetime: None,
            is_mut,
            is_unique: false,
            inner: Box::new(Type::TypeParameter("Self".to_string())),
        },
        is_static: false,
        is_mut,
    }
}

pub(super) fn identifier_expression(name: &str) -> Expression {
    Expression::new(Type::Unknown, ExpressionKind::Identifier(name.to_string()))
}

pub(super) fn string_expression(text: &str) -> Expression {
    Expression::new(
        Type::String,
        ExpressionKind::String(StringLiteral {
            parts: vec![StringPart::Text(text.to_string())],
        }),
    )
}
