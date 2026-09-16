mod search;
use inkwell::IntPredicate;
use inkwell::values;
use inkwell::values::PointerValue;

use super::super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_mir::Operand;

impl FunctionCompiler<'_, '_> {
    pub(in crate::function) fn emit_fill_i32_helper(
        &self,
        callee: &str,
        arguments: &[Operand],
    ) -> Result<(), CodeGenerationError> {
        let [destination_argument, value_argument, count_argument] =
            self.arguments(callee, arguments).exact()?;

        let destination = destination_argument.pointer("fill_i32_dest")?;
        let value = value_argument.integer(&Type::I32)?;
        let count = count_argument.integer(&Type::USize)?;
        let count = if count.get_type() == self.usize_type() {
            count
        } else {
            self.integer_cast(count, self.usize_type(), "fill_i32_count")?
        };

        self.emit_typed_slice_fill(destination, count, self.i32().into(), value.into())
    }
}
