mod alignment;
mod floating;
mod numeric;

mod lowering;

use inkwell::IntPredicate;
use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use lowering::{LlvmIntrinsicDefinition, ZeroInput};
use x_ast::Type;
use x_ast::intrinsics::{AlignOperation, NumberOperation};
use x_diagnostics::backend_messages;
use x_mir::Operand;

const BASE_PAGE_ALIGNMENT: u64 = 4 * 1024;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn emit_resolved_overflow_intrinsic(
        &self,
        function: &str,
        arguments: &[Operand],
        intrinsic: x_ast::OverflowIntrinsic,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let resolved_type = intrinsic.value_type();

        let [left, right] = self.arguments(function, arguments).exact()?;
        let left = left.integer(&resolved_type)?;
        let right = right.integer(&resolved_type)?;
        let operation = match intrinsic.operation {
            x_ast::OverflowOperation::Add => "add",
            x_ast::OverflowOperation::Subtract => "sub",
            x_ast::OverflowOperation::Multiply => "mul",
        };
        let signedness = if intrinsic.value == x_ast::OverflowType::I32 {
            "s"
        } else {
            "u"
        };

        let symbol = format!(
            "llvm.{signedness}{operation}.with.overflow.i{}",
            left.get_type().get_bit_width()
        );
        let tuple_type = self
            .context()
            .struct_type(&[left.get_type().into(), self.boolean_type().into()], false);
        let signature = tuple_type.fn_type(&[left.get_type().into(); 2], false);
        let callee = self
            .m()
            .get_function(&symbol)
            .unwrap_or_else(|| self.m().add_function(&symbol, signature, None));

        if callee.get_type() != signature {
            return Err(CodeGenerationError::InvalidMir(
                backend_messages::incompatible_overflow_intrinsic_declaration(&symbol),
            ));
        }

        self.call(
            callee,
            &[left.into(), right.into()],
            &format!("overflow_{operation}"),
        )?
        .try_as_basic_value()
        .basic()
        .ok_or(CodeGenerationError::MissingCalleeSymbol(symbol))
    }

    pub(super) fn emit_resolved_rotate_intrinsic(
        &self,
        function: &str,
        arguments: &[Operand],
        direction: x_ast::RotateDirection,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let [value, shift] = self.arguments(function, arguments).exact()?;
        let value = value.integer(&Type::U32)?;
        let shift = shift.integer(&Type::U32)?;
        let bit_width = u64::from(value.get_type().get_bit_width());
        let width = value.get_type().const_int(bit_width, false);
        let shift_mask = value.get_type().const_int(bit_width - 1, false);
        let masked = self.integer_and(shift, shift_mask, "rot_mask")?;
        let inverse = self.integer_sub(width, masked, "rot_inverse")?;
        let inverse = self.integer_and(inverse, shift_mask, "rot_inverse_masked")?;
        let left = self.shl(value, masked, "rot_left")?;
        let right = self.shr(value, inverse, false, "rot_right")?;

        let result = if direction == x_ast::RotateDirection::Left {
            self.integer_or(left, right, "rotl")?
        } else {
            let right = self.shr(value, masked, false, "rotr_right")?;
            let left = self.shl(value, inverse, "rotr_left")?;

            self.integer_or(left, right, "rotr")?
        };

        Ok(result.into())
    }

    pub(super) fn emit_resolved_bit_intrinsic(
        &self,
        current_function: &str,
        arguments: &[Operand],
        intrinsic: x_ast::BitIntrinsic,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let definition = intrinsic.llvm_definition();
        let llvm_name = definition
            .symbol
            .as_deref()
            .ok_or_else(|| CodeGenerationError::MissingCalleeSymbol(current_function.into()))?;
        let resolved_type = match intrinsic.width {
            x_ast::BitWidth::W16 => self.context().i16_type(),
            x_ast::BitWidth::W32 => self.i32(),
            x_ast::BitWidth::W64 => self.i64(),
        };

        let [value] = self.arguments(current_function, arguments).exact()?;
        let value = value.value(&intrinsic.value_type())?;
        let arity = if definition.zero_input == ZeroInput::Defined {
            2
        } else {
            1
        };

        let callee = match self.m().get_function(llvm_name) {
            Some(function) => function,
            None => {
                let parameter_types: &[_] = if arity == 2 {
                    &[resolved_type.into(), self.boolean_type().into()]
                } else {
                    &[resolved_type.into()]
                };

                let function_type = resolved_type.fn_type(parameter_types, false);

                self.m().add_function(llvm_name, function_type, None)
            }
        };

        let arguments: &[_] = if arity == 2 {
            &[value.into(), self.boolean_type().const_zero().into()]
        } else {
            &[value.into()]
        };

        let call = self.call(callee, arguments, "bit_intrinsic")?;

        call.try_as_basic_value().basic().ok_or_else(|| {
            CodeGenerationError::LlvmInternalError(
                backend_messages::bit_intrinsic_llvm_name_did_not_return_a_value(llvm_name),
            )
        })
    }
}

fn align_minmax_predicate(operation: AlignOperation) -> IntPredicate {
    match operation {
        AlignOperation::Maximum => IntPredicate::UGT,
        _ => IntPredicate::ULT,
    }
}

fn number_minmax_predicate(operation: NumberOperation) -> IntPredicate {
    match operation {
        NumberOperation::Minimum => IntPredicate::SLT,
        _ => IntPredicate::SGT,
    }
}
