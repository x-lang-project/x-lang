macro_rules! walk_statement_children {
    ($visitor:ident, $statement:expr, {$($special:tt)*}) => {
        match $statement {
            $($special)*
            $crate::Statement::Expression { expression }
            | $crate::Statement::Return { value: Some(expression) }
            | $crate::Statement::Throw { message: expression } => $visitor.visit_expression(expression),
            $crate::Statement::Assert { condition, message } => {
                $visitor.visit_expression(condition);
                if let Some(message) = message {
                    $visitor.visit_expression(message);
                }
            }
            $crate::Statement::Block { statements } => {
                $visitor.visit_statements(statements);
            }
            $crate::Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                $visitor.visit_expression(condition);
                $visitor.visit_statements(then_block);
                if let Some(else_block) = else_block {
                    $visitor.visit_statements(else_block);
                }
            }
            $crate::Statement::WhileLoop { condition, body } => {
                $visitor.visit_expression(condition);
                $visitor.visit_statements(body);
            }
            $crate::Statement::ForRangeLoop {
                start, end, body, ..
            } => {
                $visitor.visit_expression(start);
                $visitor.visit_expression(end);
                $visitor.visit_statements(body);
            }
            $crate::Statement::ForEachLoop { iterator, body, .. } => {
                $visitor.visit_expression(iterator);
                $visitor.visit_statements(body);
            }
            $crate::Statement::Become { call } => $visitor.visit_expression(call),
            $crate::Statement::Panic {
                message: Some(message),
            } => $visitor.visit_expression(message),
            $crate::Statement::Import { .. }
            | $crate::Statement::ExternFunctionDeclaration { .. }
            | $crate::Statement::Comment(_)
            | $crate::Statement::StructDeclaration(_)
            | $crate::Statement::TraitDeclaration(_)
            | $crate::Statement::ImplDeclaration(_)
            | $crate::Statement::Return { value: None }
            | $crate::Statement::Panic { message: None }
            | $crate::Statement::Break
            | $crate::Statement::Continue => {}
        }
    };
}

macro_rules! walk_expression_children {
    ($visitor:ident, $kind:expr, $iter:ident, $slice:ident, $option:ident, {$($special:tt)*}) => {
        match $kind {
            $($special)*

            $crate::ExpressionKind::String(literal) => {
                for part in literal.parts.$iter() {
                    if let $crate::StringPart::Interpolation(expression) = part {
                        $visitor.visit_expression(expression);
                    }
                }
            }
            $crate::ExpressionKind::BinaryOperator { left, right, .. }
            | $crate::ExpressionKind::ArrayRepeat {
                value: left,
                count: right,
            }
            | $crate::ExpressionKind::ArrayAccess {
                array: left,
                index: right,
            }
            | $crate::ExpressionKind::Assignment {
                target: left,
                value: right,
            } => {
                $visitor.visit_expression(left);
                $visitor.visit_expression(right);
            }
            $crate::ExpressionKind::UnaryOperator { expression, .. }
            | $crate::ExpressionKind::Try { expression }
            | $crate::ExpressionKind::AddressOf { expression, .. }
            | $crate::ExpressionKind::Dereference { expression }
            | $crate::ExpressionKind::FieldAccess { object: expression, .. } => {
                $visitor.visit_expression(expression);
            }
            $crate::ExpressionKind::FunctionCall { arguments, .. } => {
                $visitor.visit_expressions(arguments);
            }
            $crate::ExpressionKind::MethodCall { object, arguments, .. } => {
                $visitor.visit_expression(object);
                $visitor.visit_expressions(arguments);
            }
            $crate::ExpressionKind::Array(items) | $crate::ExpressionKind::Tuple(items) => {
                $visitor.visit_expressions(items);
            }
            $crate::ExpressionKind::StructInstantiate(initialise) => {
                for (_, field) in initialise.fields.$iter() {
                    $visitor.visit_expression(field);
                }
            }
            $crate::ExpressionKind::Block { statements, result } => {
                $visitor.visit_statements(statements);
                if let Some(result) = result {
                    $visitor.visit_expression(result);
                }
            }
            $crate::ExpressionKind::If {
                condition,
                then_block,
                else_block,
            } => {
                $visitor.visit_expression(condition);
                $visitor.visit_statements(then_block);
                if let Some(else_block) = else_block {
                    $visitor.visit_statements(else_block);
                }
            }
            $crate::ExpressionKind::Match { scrutinee, arms } => {
                $visitor.visit_expression(scrutinee);
                for arm in arms.$iter() {
                    $visitor.visit_statements(arm.body.$slice());
                    if let Some(result) = arm.result.$option() {
                        $visitor.visit_expression(result);
                    }
                }
            }
            $crate::ExpressionKind::Integer(_)
            | $crate::ExpressionKind::Float(_)
            | $crate::ExpressionKind::Character(_)
            | $crate::ExpressionKind::Boolean(_)
            | $crate::ExpressionKind::Identifier(_)
            | $crate::ExpressionKind::Assembly { .. } => {}
        }
    };
}
