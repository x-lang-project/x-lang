use super::literals::{check_character_literal, check_string_literal};
use super::*;

impl TypeChecker {
    pub(super) fn check_expression_inner(
        &mut self,
        expression: &mut Expression,
    ) -> Result<Type, TypeError> {
        self.check_expression_kind(expression)?
            .complete(self, expression)
    }

    fn check_expression_kind(
        &mut self,
        expression: &mut Expression,
    ) -> Result<CheckedType, TypeError> {
        let inferred = match &mut expression.kind {
            ExpressionKind::Assembly { .. } => {
                self.require_impure_operation(ImpureOperation::InlineAssembly)?;

                Ok(Type::Void)
            }
            ExpressionKind::TypeLiteral(value_type) => Ok(value_type.clone()),
            ExpressionKind::SizeOf(_) => Ok(Type::USize),
            ExpressionKind::Integer(_) => Ok(self.integer_literal_type(&expression.value_type)),
            ExpressionKind::Float(_) => Ok(self.float_literal_type(&expression.value_type)),
            ExpressionKind::String(literal) => check_string_literal(literal),
            ExpressionKind::Character(value) => check_character_literal(*value),
            ExpressionKind::Boolean(_) => Ok(Type::Boolean),

            ExpressionKind::Identifier(name) => return self.check_identifier(name),
            ExpressionKind::StructInstantiate(initialise) => {
                self.check_struct_instantiation(initialise, &expression.value_type)
            }
            ExpressionKind::FieldAccess { object, field } => {
                return self.check_field_access(object, field);
            }
            ExpressionKind::Dereference { expression: inner } => {
                self.check_dereference(inner, &mut expression.value_type)
            }
            ExpressionKind::BinaryOperator {
                left,
                operator,
                right,
            } => self.check_binary(left, operator, right),
            ExpressionKind::Assignment { target, value } => {
                self.check_assignment_expression(target, value)
            }

            ExpressionKind::Block { statements, result } => {
                self.check_scoped_block(statements, result.as_deref_mut())
            }
            ExpressionKind::If {
                condition,
                then_block,
                else_block,
            } => self.check_if_expression(condition, then_block, else_block.as_deref_mut()),
            ExpressionKind::Match { scrutinee, arms } => self.check_match(scrutinee, arms),
            ExpressionKind::Cast {
                expression: inner,
                to,
            } => self.check_cast(inner, to),

            ExpressionKind::FunctionCall { target, arguments } => {
                return self.check_function_call(target, arguments);
            }
            ExpressionKind::MethodCall {
                target,
                arguments,
                object,
            } => {
                let checked = self.check_method_call(object, target, arguments)?;

                return Ok(checked.apply_to(&mut expression.kind));
            }
            ExpressionKind::Try { expression: inner } => self.check_try(inner),

            ExpressionKind::Array(elements) => self.check_array(elements),
            ExpressionKind::ArrayRepeat { value, count } => self.check_array_repeat(value, count),
            ExpressionKind::Tuple(elements) => self.check_tuple(elements),
            ExpressionKind::AnonymousFunction {
                parameters,
                body,
                return_type,
            } => self.check_anonymous_function(parameters, body, return_type),
            ExpressionKind::ArrayAccess { array, index } => self.check_array_access(array, index),
            ExpressionKind::AddressOf {
                is_mut,
                is_unique,
                expression: inner,
            } => self.check_address_of(inner, *is_mut, *is_unique),
            ExpressionKind::UnaryOperator {
                operator,
                expression: inner,
            } => self.check_unary(operator, inner),
        };

        inferred.map(CheckedType::ResolveAnnotation)
    }
}
