use std::mem;

use super::ExpressionLowerer;
use crate::errors::MirLowerError;
use crate::{MirStatement, Operand, Place, Rvalue};
use x_ast::{Expression, Operator, Type, UnaryOperator};
use x_diagnostics::backend_messages;

impl ExpressionLowerer<'_> {
    pub(super) fn lower_integer_value(
        &mut self,
        index: i64,
        expected_type: Option<&Type>,
    ) -> Result<Operand, MirLowerError> {
        match expected_type {
            Some(expected) if expected.is_integer() && expected != &Type::I32 => Ok(self
                .emit_temporary(
                    expected.clone(),
                    Rvalue::Use(Operand::ConstantInteger(index)),
                )),
            _ => Ok(Operand::ConstantInteger(index)),
        }
    }
}
