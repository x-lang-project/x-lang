use super::{AstValidationError, Node, ValidationWork, diagnostic_path, require_name};
use crate::Statement;

pub(super) fn validate_children<'a>(
    statement: &'a Statement,
    depth: usize,
    path: &str,
    work: &mut ValidationWork<'a>,
) -> Result<(), AstValidationError> {
    let detailed = work.detailed;
    let expression = |expression, suffix: &str, work: &mut ValidationWork<'a>| {
        work.push(Node::Expression(
            expression,
            1,
            diagnostic_path(detailed, || format!("{path}.{suffix}")),
        ));
    };

    match statement {
        Statement::Expression {
            expression: inner_expression,
        } => expression(inner_expression, "expression", work),
        Statement::VariableDeclaration { value, .. } => expression(value, "initializer", work),
        Statement::Constant { name, value, .. } => {
            require_name(name, diagnostic_path(detailed, || format!("{path}.name")))?;
            expression(value, "value", work);
        }
        Statement::Import { item, .. } => {
            require_name(item, diagnostic_path(detailed, || format!("{path}.item")))?
        }
        Statement::TypeAlias { name, .. } => {
            require_name(name, diagnostic_path(detailed, || format!("{path}.name")))?;
        }
        Statement::Function {
            name,
            parameters,
            body,
            ..
        } => {
            require_name(name, diagnostic_path(detailed, || format!("{path}.name")))?;
            for (index, parameter) in parameters.iter().enumerate() {
                let parameter_path =
                    diagnostic_path(detailed, || format!("{path}.parameter[{index}]"));
                require_name(&parameter.name, parameter_path.clone())?;

                if parameter.is_receiver() && index != 0 {
                    return Err(AstValidationError::ReceiverPosition {
                        path: parameter_path,
                    });
                }
            }

            if let Some(body) = body {
                work.push_statements(body, depth + 1, path);
            }
        }
        Statement::ExternFunctionDeclaration { name, .. } => {
            require_name(name, diagnostic_path(detailed, || format!("{path}.name")))?;
        }
        Statement::Assert { condition, message } => {
            expression(condition, "condition", work);

            if let Some(message) = message {
                expression(message, "message", work);
            }
        }
        Statement::Panic { message } | Statement::Return { value: message } => {
            if let Some(message) = message {
                expression(message, "value", work);
            }
        }
        Statement::Throw { message } => expression(message, "message", work),
        Statement::Block { statements } => work.push_statements(statements, depth + 1, path),
        Statement::Become { call } => expression(call, "call", work),
        Statement::ForRangeLoop {
            variable,
            start,
            end,
            body,
            ..
        } => {
            require_name(
                variable,
                diagnostic_path(detailed, || format!("{path}.variable")),
            )?;
            expression(start, "start", work);
            expression(end, "end", work);
            work.push_statements(body, depth + 1, path);
        }
        Statement::If {
            condition,
            then_block,
            else_block,
        } => {
            expression(condition, "condition", work);
            work.push_statements(then_block, depth + 1, path);

            if let Some(else_block) = else_block {
                work.push_statements(else_block, depth + 1, path);
            }
        }
        Statement::WhileLoop { condition, body } => {
            expression(condition, "condition", work);
            work.push_statements(body, depth + 1, path);
        }
        Statement::ForEachLoop {
            variable,
            iterator,
            body,
            ..
        } => {
            require_name(
                variable,
                diagnostic_path(detailed, || format!("{path}.variable")),
            )?;
            expression(iterator, "iterator", work);
            work.push_statements(body, depth + 1, path);
        }
        Statement::StructDeclaration(definition) => {
            require_name(
                &definition.name,
                diagnostic_path(detailed, || format!("{path}.name")),
            )?;
            for (index, (field, _)) in definition.fields.iter().enumerate() {
                require_name(
                    field,
                    diagnostic_path(detailed, || format!("{path}.field[{index}]")),
                )?;
            }

            if let Some(invariant) = &definition.invariant {
                require_name(
                    &invariant.receiver.name,
                    diagnostic_path(detailed, || format!("{path}.invariant.receiver")),
                )?;
                expression(&invariant.predicate, "invariant", work);
            }
        }
        Statement::TraitDeclaration(definition) => {
            require_name(
                &definition.name,
                diagnostic_path(detailed, || format!("{path}.name")),
            )?;

            for (name, method) in &definition.methods {
                require_name(name, diagnostic_path(detailed, || format!("{path}.method")))?;

                if let Some(method) = method {
                    work.push(Node::Statement(
                        method,
                        depth + 1,
                        diagnostic_path(detailed, || format!("{path}.method.{name}")),
                    ));
                }
            }
        }
        Statement::ImplDeclaration(definition) => {
            work.push_statements(&definition.methods, depth + 1, path);
        }
        Statement::Comment(_) | Statement::Break | Statement::Continue => {}
    }

    Ok(())
}
