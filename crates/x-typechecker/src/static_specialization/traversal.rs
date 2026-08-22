use super::*;

#[derive(Clone, Copy)]
pub(super) enum DeclarationBodies {
    Enter,
    Skip,
}

pub(super) fn visit_statement_expressions_mut(
    statement: &mut Statement,
    declarations: DeclarationBodies,
    visit: &mut impl FnMut(&mut Expression) -> Result<(), TypeError>,
) -> Result<(), TypeError> {
    match statement {
        Statement::Expression { expression }
        | Statement::Become { call: expression }
        | Statement::Throw {
            message: expression,
        } => visit(expression),
        Statement::VariableDeclaration { value, .. } | Statement::Constant { value, .. } => {
            visit(value)
        }
        Statement::Return { value } | Statement::Panic { message: value } => {
            if let Some(value) = value {
                visit(value)?;
            }

            Ok(())
        }
        Statement::Assert { condition, message } => {
            visit(condition)?;
            if let Some(message) = message {
                visit(message)?;
            }

            Ok(())
        }
        Statement::Block { statements } => {
            for statement in statements {
                visit_statement_expressions_mut(statement, declarations, visit)?;
            }

            Ok(())
        }
        Statement::If {
            condition,
            then_block,
            else_block,
        } => {
            visit(condition)?;
            for current_statement in then_block {
                visit_statement_expressions_mut(current_statement, declarations, visit)?;
            }

            if let Some(block) = else_block {
                for current_statement in block {
                    visit_statement_expressions_mut(current_statement, declarations, visit)?;
                }
            }

            Ok(())
        }
        Statement::WhileLoop { condition, body } => {
            visit(condition)?;
            for current_statement in body {
                visit_statement_expressions_mut(current_statement, declarations, visit)?;
            }

            Ok(())
        }
        Statement::ForRangeLoop {
            start, end, body, ..
        } => {
            visit(start)?;
            visit(end)?;
            for current_statement in body {
                visit_statement_expressions_mut(current_statement, declarations, visit)?;
            }

            Ok(())
        }
        Statement::ForEachLoop { iterator, body, .. } => {
            visit(iterator)?;
            for current_statement in body {
                visit_statement_expressions_mut(current_statement, declarations, visit)?;
            }

            Ok(())
        }
        Statement::Function { body, .. } if matches!(declarations, DeclarationBodies::Enter) => {
            if let Some(body) = body {
                for statement in body.iter_mut() {
                    visit_statement_expressions_mut(statement, declarations, visit)?;
                }
            }

            Ok(())
        }
        Statement::ImplDeclaration(definition)
            if matches!(declarations, DeclarationBodies::Enter) =>
        {
            for method in &mut definition.methods {
                visit_statement_expressions_mut(method, declarations, visit)?;
            }

            Ok(())
        }
        Statement::TraitDeclaration(definition)
            if matches!(declarations, DeclarationBodies::Enter) =>
        {
            for method in definition.methods.values_mut().flatten() {
                visit_statement_expressions_mut(method, declarations, visit)?;
            }

            Ok(())
        }
        Statement::Function { .. } => Ok(()),
        Statement::StructDeclaration(definition) => {
            if let Some(inv) = &mut definition.invariant {
                visit(&mut inv.predicate)
            } else {
                Ok(())
            }
        }
        Statement::TypeAlias { .. } => Ok(()),
        Statement::ImplDeclaration(_) | Statement::TraitDeclaration(_) => Ok(()),
        Statement::ExternFunctionDeclaration { .. }
        | Statement::Import { .. }
        | Statement::Comment(_)
        | Statement::Break
        | Statement::Continue => Ok(()),
    }
}

pub(super) fn walk_expression_children_mut(
    expression: &mut Expression,
    visit: &mut impl FnMut(&mut Expression) -> Result<(), TypeError>,
) -> Result<(), TypeError> {
    match &mut expression.kind {
        ExpressionKind::Cast { expression, .. }
        | ExpressionKind::Try { expression }
        | ExpressionKind::UnaryOperator { expression, .. }
        | ExpressionKind::AddressOf { expression, .. }
        | ExpressionKind::Dereference { expression } => visit(expression)?,
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
            visit(left)?;
            visit(right)?;
        }
        ExpressionKind::FunctionCall { arguments, .. } => {
            for argument in arguments {
                visit(argument)?;
            }
        }
        ExpressionKind::MethodCall {
            object, arguments, ..
        } => {
            visit(object)?;

            for argument in arguments {
                visit(argument)?;
            }
        }
        ExpressionKind::Tuple(items) | ExpressionKind::Array(items) => {
            for item in items {
                visit(item)?;
            }
        }
        ExpressionKind::StructInstantiate(initialise) => {
            for (_, value) in &mut initialise.fields {
                visit(value)?;
            }
        }
        ExpressionKind::FieldAccess { object, .. } => visit(object)?,
        ExpressionKind::Block { statements, result } => {
            for statement in statements {
                visit_statement_expressions_mut(statement, DeclarationBodies::Skip, visit)?;
            }

            if let Some(result) = result {
                visit(result)?;
            }
        }
        ExpressionKind::If {
            condition,
            then_block,
            else_block,
        } => {
            visit(condition)?;
            for current_statement in then_block {
                visit_statement_expressions_mut(current_statement, DeclarationBodies::Skip, visit)?;
            }

            if let Some(block) = else_block {
                for current_statement in block {
                    visit_statement_expressions_mut(
                        current_statement,
                        DeclarationBodies::Skip,
                        visit,
                    )?;
                }
            }
        }
        ExpressionKind::Match { scrutinee, arms } => {
            visit(scrutinee)?;

            for arm in arms {
                for current_statement in &mut arm.body {
                    visit_statement_expressions_mut(
                        current_statement,
                        DeclarationBodies::Skip,
                        visit,
                    )?;
                }

                if let Some(result) = &mut arm.result {
                    visit(result)?;
                }
            }
        }
        ExpressionKind::AnonymousFunction { .. } => {}
        ExpressionKind::String(literal) => {
            for part in &mut literal.parts {
                if let StringPart::Interpolation(expression) = part {
                    visit(expression)?;
                }
            }
        }
        ExpressionKind::Integer(_)
        | ExpressionKind::Float(_)
        | ExpressionKind::Character(_)
        | ExpressionKind::Boolean(_)
        | ExpressionKind::Identifier(_)
        | ExpressionKind::TypeLiteral(_)
        | ExpressionKind::SizeOf(_)
        | ExpressionKind::Assembly { .. } => {}
    }

    Ok(())
}
