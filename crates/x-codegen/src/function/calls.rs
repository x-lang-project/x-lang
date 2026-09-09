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

    fn resolve_unique_suffix_callee(
        &self,
        base_name: &str,
        arity: Option<usize>,
    ) -> Option<values::FunctionValue<'ctx>> {
        if let Some(function) = self
            .codegen
            .resolve_function_by_source_suffix(base_name, arity)
        {
            return Some(function);
        }

        let suffix = format!("::{base_name}");
        let mut matches = self.m().get_functions().filter(|function| {
            function
                .get_name()
                .to_str()
                .ok()
                .is_some_and(|name| name == base_name || name.ends_with(&suffix))
                && function_matches_arity(*function, arity)
        });

        let first = matches.next()?;

        matches.next().is_none().then_some(first)
    }
}
