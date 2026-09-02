use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::values::BasicValueEnum;
use inkwell::{FloatPredicate, IntPredicate};

use crate::errors::CodeGenerationError;
use x_ast::Operator;
use x_diagnostics::backend_messages;

#[derive(Clone, Copy)]
pub enum IntegerSignedness {
    Signed,
    Unsigned,
}

impl IntegerSignedness {
    fn is_signed(self) -> bool {
        matches!(self, Self::Signed)
    }
}

pub struct OperatorEmitter<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
}

impl<'ctx> OperatorEmitter<'_, 'ctx> {
    pub(crate) fn normalize_binary_operands(
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, BasicValueEnum<'ctx>), CodeGenerationError> {
        if left.get_type() == right.get_type() {
            Ok((left, right))
        } else {
            Err(CodeGenerationError::LlvmInternalError(backend_messages::type_mismatch_invariant_violated_before_operator_emission_value_vs_value(left.get_type(), right.get_type())))
        }
    }
}
