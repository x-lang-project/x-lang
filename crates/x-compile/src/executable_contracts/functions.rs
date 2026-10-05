use super::*;

pub(super) fn validate_function(
    statement: &Statement,
    functions: &HashMap<String, FunctionShape>,
    structures: &HashMap<String, Vec<Type>>,
) -> Result<(), CompilationFailure> {
    let Statement::Function {
        name,
        parameters,
        return_type,
        body,
        is_pure,
        is_memoised,
        is_multi,
        is_throws,
        generic_parameters,
        ..
    } = statement
    else {
        return Ok(());
    };

    if *is_pure && *is_throws {
        return Err(contract_messages::pure_function_throws(name));
    }

    if *is_memoised {
        if !*is_pure {
            return Err(contract_messages::memoized_function_impure(name));
        }

        if *is_throws {
            return Err(contract_messages::memoized_function_throws(name));
        }

        if *is_multi {
            return Err(contract_messages::memoized_multi_function(name));
        }

        let scalar_or_parameter = |value_type: &Type| {
            value_type.is_scalar()
                || matches!(value_type, Type::TypeParameter(name) | Type::Custom(name)
                    if generic_parameters.iter().any(|parameter|
                        matches!(parameter, x_ast::GenericParameter::Type { name: declared, .. }
                            if declared == name)))
        };

        if parameters
            .iter()
            .any(|pointer| !scalar_or_parameter(&pointer.value_type))
            || !scalar_or_parameter(return_type)
        {
            return Err(contract_messages::memoized_function_non_scalar(name));
        }
    }

    if *is_multi && (parameters.is_empty() || parameters.iter().any(|pointer| pointer.is_static)) {
        return Err(contract_messages::multi_function_dispatch_missing(name));
    }

    if let Some(body) = body {
        validate_statements(body, functions, structures, false)?;
    }

    Ok(())
}

pub(super) fn validate_parallel_range(
    variable: &str,
    body: &[Statement],
    functions: &HashMap<String, FunctionShape>,
    structures: &HashMap<String, Vec<Type>>,
) -> Result<(), CompilationFailure> {
    let mut executable = body
        .iter()
        .filter(|statement| !matches!(statement, Statement::Comment(_)));
    let (Some(Statement::Expression { expression }), None) = (executable.next(), executable.next())
    else {
        return Err(contract_messages::parallel_body_unsupported());
    };

    let ExpressionKind::FunctionCall { target, arguments } = &expression.kind else {
        return Err(contract_messages::parallel_body_unsupported());
    };

    let name = target.symbol_name();

    if !matches!(arguments.first(), Some(argument) if matches!(&argument.kind, ExpressionKind::Identifier(input) if input == variable))
    {
        return Err(contract_messages::parallel_worker_input_invalid());
    }

    let Some(shape) = functions.get(name.as_ref()) else {
        return Err(contract_messages::parallel_worker_unknown(&name));
    };

    if !shape.pure
        || shape.throws
        || shape.parameters.first() != Some(&Type::I32)
        || shape.parameters.len() != arguments.len()
        || shape.result != Type::I32
    {
        return Err(contract_messages::parallel_worker_signature_invalid(&name));
    }

    for (argument, parameter) in arguments.iter().zip(&shape.parameters).skip(1) {
        if !invariant_capture(argument, variable)
            || !shareable_capture(parameter, structures)
            || !shareable_capture(&argument.value_type, structures)
        {
            return Err(contract_messages::parallel_capture_unsupported());
        }
    }

    Ok(())
}

fn invariant_capture(expression: &Expression, iteration_variable: &str) -> bool {
    struct Dependencies<'a> {
        iteration_variable: &'a str,
        reads_iteration: bool,
    }

    impl x_ast::ExpressionVisitor for Dependencies<'_> {
        fn visit_expression(&mut self, expression: &Expression) {
            if matches!(&expression.kind, ExpressionKind::Identifier(name)
                if name == self.iteration_variable)
            {
                self.reads_iteration = true;
            }

            if !self.reads_iteration {
                x_ast::walk_expression_children(self, expression);
            }
        }
    }

    let mut dependencies = Dependencies {
        iteration_variable,
        reads_iteration: false,
    };
    x_ast::ExpressionVisitor::visit_expression(&mut dependencies, expression);

    !dependencies.reads_iteration
}

pub(super) fn executable_invariant(invariant: &x_ast::StructInvariant) -> bool {
    executable_invariant_expression(&invariant.predicate, &invariant.receiver)
}

fn executable_invariant_expression(
    expression: &Expression,
    receiver: &x_ast::InvariantBinding,
) -> bool {
    match &expression.kind {
        ExpressionKind::Integer(_)
        | ExpressionKind::Float(_)
        | ExpressionKind::Boolean(_)
        | ExpressionKind::Character(_) => true,
        ExpressionKind::Identifier(_) => receiver.refers_to(expression),
        ExpressionKind::FieldAccess { object, .. }
        | ExpressionKind::UnaryOperator {
            expression: object, ..
        } => executable_invariant_expression(object, receiver),
        ExpressionKind::BinaryOperator { left, right, .. } => {
            executable_invariant_expression(left, receiver)
                && executable_invariant_expression(right, receiver)
        }
        ExpressionKind::Cast { expression, to } => {
            expression.value_type == *to && executable_invariant_expression(expression, receiver)
        }
        _ => false,
    }
}
