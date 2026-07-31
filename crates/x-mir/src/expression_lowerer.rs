use std::collections::{HashMap, HashSet};

use crate::errors::MirLowerError;
use crate::local_declarations::LocalDeclarations;
use crate::lower;
use crate::{LocalIdentifier, MirStatement, Operand, Place, Rvalue};
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::backend_messages;
use x_diagnostics::mir_messages;

mod aggregates;
mod calls;
pub(crate) use calls::PreparedCall;
mod helpers;
mod memory;
mod places;
mod values;

pub(crate) struct ExpressionLowerer<'a> {
    pub(crate) locals: &'a mut LocalDeclarations,
    pub(crate) variable_mappings: &'a HashMap<String, LocalIdentifier>,
    pub(crate) global_mappings: &'a HashMap<String, Type>,
    pub(crate) source_functions: &'a HashSet<String>,
    pub(crate) statements_buffer: &'a mut Vec<MirStatement>,
}

impl ExpressionLowerer<'_> {
    fn allocate_temporary(&mut self, value_type: Type) -> LocalIdentifier {
        self.locals.allocate(None, value_type)
    }

    fn emit_statement(&mut self, statement: MirStatement) {
        self.statements_buffer.push(statement);
    }

    fn emit_temporary(&mut self, value_type: Type, value: Rvalue) -> Operand {
        let local = self.allocate_temporary(value_type);
        self.emit_statement(MirStatement::Assign(Place::Local(local), value));

        Operand::Move(Box::new(Place::Local(local)))
    }

    pub fn lower_expression_with_context(
        &mut self,
        expression: Expression,
        expected_type: Option<&Type>,
    ) -> Result<Operand, MirLowerError> {
        let checked_type = lower::resolved_expression_type(&expression);

        match expression.kind {
            ExpressionKind::Integer(position) => {
                self.lower_integer_value(position, expected_type.or(checked_type.as_ref()))
            }
            ExpressionKind::Float(function) => {
                let value = Operand::ConstantFloat(function);

                if expected_type.or(checked_type.as_ref()) == Some(&Type::F32) {
                    Ok(self.emit_temporary(Type::F32, Rvalue::Use(value)))
                } else {
                    Ok(value)
                }
            }
            ExpressionKind::Boolean(second_value) => Ok(Operand::ConstantBoolean(second_value)),
            ExpressionKind::Character(character) => Ok(Operand::ConstantCharacter(character)),
            ExpressionKind::String(literal) => crate::lower::literal_string_value(&literal)
                .map(Operand::ConstantString)
                .ok_or_else(|| {
                    MirLowerError::UnsupportedExpression(
                        mir_messages::runtime_string_interpolation_required(),
                    )
                }),
            ExpressionKind::Identifier(name) => self
                .resolve_identifier_place(name)
                .map(|place| Operand::Copy(Box::new(place))),
            ExpressionKind::BinaryOperator {
                left,
                operator,
                right,
            } => self.lower_binary_value(*left, operator, *right, expression.value_type),
            ExpressionKind::UnaryOperator {
                operator,
                expression: inner_expression,
            } => {
                let inner_operator =
                    self.lower_expression_with_context(*inner_expression, expected_type)?;

                Ok(self.emit_temporary(
                    expression.value_type,
                    Rvalue::UnaryOperator(operator, inner_operator),
                ))
            }
            ExpressionKind::Cast {
                expression: inner_expression,
                to,
            } => {
                let inner_operator = self.lower_expression_with_context(*inner_expression, None)?;

                Ok(self.emit_temporary(
                    to.clone(),
                    Rvalue::Cast {
                        expression: inner_operator,
                        to,
                    },
                ))
            }
            ExpressionKind::AddressOf {
                is_mut,
                is_unique,
                expression: inner_expression,
            } => self.lower_reference_value(
                *inner_expression,
                x_ast::ReferenceQualifiers { is_mut, is_unique },
                expected_type.or(checked_type.as_ref()),
            ),
            ExpressionKind::Dereference {
                expression: inner_expression,
            } => self.lower_dereference_value(*inner_expression, expression.value_type),
            ExpressionKind::ArrayAccess { array, index } => {
                self.lower_index_value(*array, *index, expression.value_type)
            }
            ExpressionKind::FieldAccess { object, field } => {
                self.lower_field_value(*object, field, expression.value_type)
            }
            ExpressionKind::FunctionCall { target, arguments } => self.lower_function_call(
                target.runtime_spelling().into_owned(),
                arguments,
                expression.value_type,
            ),
            ExpressionKind::MethodCall {
                object,
                target,
                arguments,
            } => self.lower_method_call(
                *object,
                target.runtime_spelling().into_owned(),
                arguments,
                expression.value_type,
            ),
            ExpressionKind::Try {
                expression: inner_expression,
            } => self.lower_try_value(*inner_expression, expected_type),
            ExpressionKind::Assignment { target, value } => {
                self.lower_assignment_value(*target, *value)
            }
            ExpressionKind::TypeLiteral(value_type) => Ok(Operand::TypeLiteral(value_type)),
            ExpressionKind::SizeOf(value_type) => {
                Ok(self.emit_temporary(Type::USize, Rvalue::SizeOf(value_type)))
            }
            ExpressionKind::StructInstantiate(initialise) => {
                self.lower_struct_value(initialise, expected_type.or(checked_type.as_ref()))
            }
            ExpressionKind::Tuple(elements) => {
                self.lower_tuple_value(elements, expression.value_type)
            }
            ExpressionKind::Array(elements) => {
                self.lower_array_value(elements, expression.value_type, expected_type)
            }
            ExpressionKind::ArrayRepeat { value, count } => self.lower_repeated_array_value(
                *value,
                *count,
                expected_type.or(checked_type.as_ref()),
            ),
            ExpressionKind::Match { .. } => Err(MirLowerError::UnsupportedExpression(
                mir_messages::statement_aware_match_required(),
            )),
            ExpressionKind::If {
                condition,
                then_block,
                else_block,
            } => {
                if let Some(value) =
                    lower::constant_branch_value(&condition, &then_block, else_block.as_deref())
                {
                    return self.lower_expression_with_context(value.clone(), expected_type);
                }

                Err(MirLowerError::UnsupportedExpression(
                    backend_messages::if_expression_lowering_requires_expression_valued_branches(),
                ))
            }
            other => Err(MirLowerError::UnsupportedExpression(
                backend_messages::expression_lowering_is_not_implemented_for_this_shape_other(
                    other,
                ),
            )),
        }
    }
}
