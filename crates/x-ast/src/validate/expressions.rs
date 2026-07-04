use super::{AstValidationError, Node, ValidationWork, diagnostic_path, require_name};
use crate::{Expression, ExpressionKind, StringPart};

pub(super) fn validate_children<'a>(
    expression: &'a Expression,
    depth: usize,
    path: &str,
    work: &mut ValidationWork<'a>,
) -> Result<(), AstValidationError> {
    let detailed = work.detailed;
    let child = |expression, suffix: &str, work: &mut ValidationWork<'a>| {
        work.push(Node::Expression(
            expression,
            depth + 1,
            diagnostic_path(detailed, || format!("{path}.{suffix}")),
        ));
    };

    match &expression.kind {
        ExpressionKind::Float(value) if !value.is_finite() => {
            return Err(AstValidationError::NonFiniteFloat {
                path: path.to_string(),
            });
        }
        ExpressionKind::Identifier(name) => {
            require_name(name, diagnostic_path(detailed, || format!("{path}.name")))?
        }
        ExpressionKind::Cast {
            expression: inner_expression,
            ..
        }
        | ExpressionKind::Try {
            expression: inner_expression,
        }
        | ExpressionKind::UnaryOperator {
            expression: inner_expression,
            ..
        }
        | ExpressionKind::AddressOf {
            expression: inner_expression,
            ..
        }
        | ExpressionKind::Dereference {
            expression: inner_expression,
        } => child(inner_expression, "operand", work),
        ExpressionKind::Tuple(elements) | ExpressionKind::Array(elements) => {
            work.push_expressions(elements, depth + 1, path);
        }
        ExpressionKind::BinaryOperator { left, right, .. }
        | ExpressionKind::ArrayRepeat {
            value: left,
            count: right,
        }
        | ExpressionKind::ArrayAccess {
            array: left,
            index: right,
        }
        | ExpressionKind::Assignment {
            target: left,
            value: right,
        } => {
            child(left, "left", work);
            child(right, "right", work);
        }
        ExpressionKind::FunctionCall {
            target: name,
            arguments,
        } => {
            validate_target(name, diagnostic_path(detailed, || format!("{path}.name")))?;
            work.push_expressions(arguments, depth + 1, path);
        }
        ExpressionKind::MethodCall {
            object,
            target: method,
            arguments,
        } => {
            validate_target(
                method,
                diagnostic_path(detailed, || format!("{path}.method")),
            )?;
            child(object, "object", work);
            work.push_expressions(arguments, depth + 1, path);
        }
        ExpressionKind::AnonymousFunction {
            parameters, body, ..
        } => {
            for (index, parameter) in parameters.iter().enumerate() {
                require_name(
                    parameter,
                    diagnostic_path(detailed, || format!("{path}.parameter[{index}]")),
                )?;
            }

            work.push_statements(body, 1, path);
        }
        ExpressionKind::StructInstantiate(initializer) => {
            require_name(
                &initializer.name,
                diagnostic_path(detailed, || format!("{path}.name")),
            )?;

            for (index, (field, value)) in initializer.fields.iter().enumerate() {
                require_name(
                    field,
                    diagnostic_path(detailed, || format!("{path}.field[{index}]")),
                )?;
                child(value, "field_value", work);
            }
        }
        ExpressionKind::FieldAccess { object, field } => {
            require_name(field, diagnostic_path(detailed, || format!("{path}.field")))?;
            child(object, "object", work);
        }
        ExpressionKind::Block { statements, result } => {
            work.push_statements(statements, 1, path);

            if let Some(result) = result {
                child(result, "result", work);
            }
        }
        ExpressionKind::If {
            condition,
            then_block,
            else_block,
        } => {
            child(condition, "condition", work);
            work.push_statements(then_block, 1, path);

            if let Some(else_block) = else_block {
                work.push_statements(else_block, 1, path);
            }
        }
        ExpressionKind::Match { scrutinee, arms } => {
            child(scrutinee, "scrutinee", work);

            for (index, arm) in arms.iter().enumerate() {
                work.push_statements(
                    &arm.body,
                    1,
                    &diagnostic_path(detailed, || format!("{path}.arm[{index}]")),
                );

                if let Some(result) = &arm.result {
                    child(result, "arm_result", work);
                }
            }
        }
        ExpressionKind::String(literal) => {
            for (index, part) in literal.parts.iter().enumerate() {
                if let StringPart::Interpolation(interpolation) = part {
                    child(
                        interpolation,
                        &diagnostic_path(detailed, || format!("interpolation[{index}]")),
                        work,
                    );
                }
            }
        }
        ExpressionKind::Integer(_)
        | ExpressionKind::Float(_)
        | ExpressionKind::Character(_)
        | ExpressionKind::Boolean(_)
        | ExpressionKind::TypeLiteral(_)
        | ExpressionKind::SizeOf(_)
        | ExpressionKind::Assembly { .. } => {}
    }

    Ok(())
}

fn validate_target(target: &crate::CallTarget, path: String) -> Result<(), AstValidationError> {
    if target.path.segments.is_empty() {
        require_name("", path.clone())?;
    }

    for segment in &target.path.segments {
        require_name(&segment.name, path.clone())?;
    }

    if let Some(name) = target.resolved_name() {
        require_name(name, path)?;
    }

    Ok(())
}
