mod casts;
use inkwell::IntPredicate;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use super::numeric;
use super::numeric::is_signed_integer_type;
use crate::errors::CodeGenerationError;
use crate::function_helpers::*;
use x_ast::{Operator, Type, UnaryOperator};
use x_diagnostics::backend_messages;
use x_mir::{Operand, Place, Rvalue};

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    fn load_binary_operand(
        &self,
        operand: &Operand,
        operand_type: Option<&Type>,
        other_type: Option<&Type>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        if let Some(value_type) = other_type
            .filter(|value_type| should_coerce_integer_operand(operand, operand_type, value_type))
        {
            self.load_operand_value_with_type(operand, value_type)
        } else {
            self.load_operand_value(operand)
        }
    }

    pub(super) fn compile_rvalue(
        &self,
        rvalue: &Rvalue,
        target_place: &Place,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        match rvalue {
            Rvalue::TryCall { .. } | Rvalue::CatchCall { .. } => {
                Err(CodeGenerationError::LlvmInternalError(
                    backend_messages::checked_call_remained_a_statement_instead_of_a_mir_terminator(
                    ),
                ))
            }
            Rvalue::Use(operand) => {
                let target_ast_type = match self.get_place_type(target_place) {
                    Type::Infer(_) | Type::Unknown => {
                        self.operand_ast_type(operand).unwrap_or(Type::Unknown)
                    }
                    value_type => value_type,
                };

                self.load_operand_value_with_type(operand, &target_ast_type)
            }
            Rvalue::BinaryOperator(operator, left, right) => {
                self.compile_binary_rvalue(operator, left, right)
            }
            Rvalue::UnaryOperator(operator, operand) => {
                self.compile_unary_rvalue(operator, operand)
            }
            Rvalue::PointerOffset { base, offset } => self.compile_pointer_offset(base, offset),
            Rvalue::AddressOf {
                is_mut,
                is_unique,
                expression,
            } => {
                let pointer_location = self.resolve_place_pointer(expression)?;

                self.constrain_safe_reference(
                    pointer_location,
                    expression,
                    x_ast::ReferenceQualifiers {
                        is_mut: *is_mut,
                        is_unique: *is_unique,
                    },
                )
                .map(BasicValueEnum::PointerValue)
            }
            Rvalue::Cast { expression, to } => self.compile_cast_rvalue(expression, to),
            Rvalue::SizeOf(value_type) => {
                let llvm_type = self.codegen.resolve_llvm_type(value_type)?;
                let size_value = llvm_type
                    .size_of()
                    .ok_or_else(|| CodeGenerationError::TypeHasNoSize(value_type.clone()))?;

                Ok(size_value.into())
            }
            Rvalue::Call {
                function,
                arguments,
            } => self
                .emit_call_value(
                    function,
                    arguments,
                    Some(&self.get_place_type(target_place)),
                )?
                .ok_or_else(|| {
                    CodeGenerationError::LlvmInternalError(
                        backend_messages::void_call_function_used_where_a_value_was_required(
                            function,
                        ),
                    )
                }),
            Rvalue::StructInstantiate { name, fields } => self.compile_struct_rvalue(name, fields),
            Rvalue::Tuple(elements) => self.compile_tuple_rvalue(elements, target_place),
            Rvalue::Array(elements) => self.compile_array_rvalue(elements, target_place),
        }
    }

    fn compile_binary_rvalue(
        &self,
        operator: &Operator,
        left: &Operand,
        right: &Operand,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let left_type = self.operand_ast_type(left);
        let right_type = self.operand_ast_type(right);
        let current_left =
            self.load_binary_operand(left, left_type.as_ref(), right_type.as_ref())?;
        let current_right =
            self.load_binary_operand(right, right_type.as_ref(), left_type.as_ref())?;
        let pointee_type = left_type
            .as_ref()
            .and_then(Type::pointee)
            .or_else(|| right_type.as_ref().and_then(Type::pointee));

        let offset_type = if current_left.is_pointer_value() {
            right_type.as_ref()
        } else {
            left_type.as_ref()
        }
        .unwrap_or(&Type::Unknown);

        if let Some(value) = self.emit_pointer_arithmetic(
            operator,
            current_left,
            current_right,
            crate::function::coercions::PointerArithmeticTypes {
                pointee: pointee_type,
                offset: offset_type,
            },
        )? {
            return Ok(value);
        }

        let operation_left = right_type
            .as_ref()
            .filter(|value_type| {
                should_coerce_integer_operand(left, left_type.as_ref(), value_type)
            })
            .or(left_type.as_ref())
            .unwrap_or(&Type::Unknown);
        let operation_right = left_type
            .as_ref()
            .filter(|value_type| {
                should_coerce_integer_operand(right, right_type.as_ref(), value_type)
            })
            .or(right_type.as_ref())
            .unwrap_or(&Type::Unknown);

        self.emit_typed_binary(
            operator,
            numeric::TypedValue {
                value: current_left,
                value_type: operation_left,
            },
            numeric::TypedValue {
                value: current_right,
                value_type: operation_right,
            },
        )
    }

    fn compile_unary_rvalue(
        &self,
        operator: &UnaryOperator,
        operand: &Operand,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let element = self.load_operand_value(operand)?;

        match operator {
            UnaryOperator::Negate => {
                if element.is_float_value() {
                    self.float_neg(element.into_float_value(), "fneg_tmp")
                        .map(|value| value.into())
                } else {
                    self.integer_neg(element.into_int_value(), "ineg_tmp")
                        .map(|value| value.into())
                }
            }
            UnaryOperator::LogicalNot => {
                let integer_value = element.into_int_value();
                let zero = integer_value.get_type().const_zero();

                self.icmp(IntPredicate::EQ, integer_value, zero, "logical_not")
                    .map(|value| value.into())
            }
            UnaryOperator::BitwiseNot => self
                .integer_not(element.into_int_value(), "not_tmp")
                .map(|value| value.into()),
            _ => Err(CodeGenerationError::UnsupportedUnaryOperator {
                operator: operator.clone(),
            }),
        }
    }

    fn compile_pointer_offset(
        &self,
        base: &Operand,
        offset: &Operand,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let base_pointer = self.load_operand_value(base)?.into_pointer_value();
        let offset_value = self
            .load_operand_value_with_type(offset, &Type::USize)?
            .into_int_value();
        let pointer_ast_type = self
            .operand_ast_type(base)
            .ok_or(CodeGenerationError::UnsupportedType(Type::Unknown))?;
        let pointee_ast_type = pointer_ast_type
            .pointee()
            .ok_or_else(|| CodeGenerationError::UnsupportedType(pointer_ast_type.clone()))?;

        let element_llvm_type = self.codegen.resolve_llvm_type(pointee_ast_type)?;

        self.gep(
            element_llvm_type,
            base_pointer,
            offset_value,
            "ptr_arith_gep",
        )
        .map(|value| value.into())
    }
}

fn should_coerce_integer_operand(
    operand: &Operand,
    operand_type: Option<&Type>,
    peer_type: &Type,
) -> bool {
    let needs_concrete_type = matches!(operand, Operand::ConstantInteger(_))
        || operand_type.is_some_and(is_generic_placeholder_type)
        || operand_type
            .is_some_and(|value_type| matches!(value_type, Type::Unknown | Type::Infer(_)));

    needs_concrete_type && is_integer_ast_type(peer_type)
}
