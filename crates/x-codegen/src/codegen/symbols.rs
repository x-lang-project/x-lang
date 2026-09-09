use inkwell::values::FunctionValue;

use super::CodeGeneration;
use crate::errors::CodeGenerationError;
use crate::function_helpers::function_matches_arity;
use crate::symbols::{MangleConfig, collect_symbol_table};
use x_diagnostics::backend_messages;
use x_mir::MirProgram;

impl<'ctx> CodeGeneration<'ctx> {
    pub fn resolve_function_by_source_suffix(
        &self,
        base_name: &str,
        arity: Option<usize>,
    ) -> Option<FunctionValue<'ctx>> {
        if base_name.contains("::") {
            let llvm_name = self.function_symbols.get(base_name)?;

            return self
                .module
                .get_function(llvm_name)
                .filter(|function| function_matches_arity(*function, arity));
        }

        let suffix = format!("::{base_name}");
        let mut matches = self
            .function_symbols
            .iter()
            .filter(|(source_name, _)| {
                source_name.as_str() == base_name || source_name.ends_with(&suffix)
            })
            .filter_map(|(_, llvm_name)| self.module.get_function(llvm_name))
            .filter(|function| function_matches_arity(*function, arity));

        let first = matches.next()?;

        matches.next().is_none().then_some(first)
    }
}
