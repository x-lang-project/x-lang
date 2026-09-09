use inkwell::values;

use super::FunctionCompiler;

use crate::errors::CodeGenerationError;
use crate::function_helpers::function_matches_arity;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn required_function(
        &self,
        name: &str,
    ) -> Result<values::FunctionValue<'ctx>, CodeGenerationError> {
        self.m()
            .get_function(name)
            .ok_or_else(|| CodeGenerationError::MissingCalleeSymbol(name.to_string()))
    }
}
