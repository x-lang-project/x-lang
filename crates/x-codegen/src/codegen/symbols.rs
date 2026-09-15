use std::collections;

use inkwell::values::FunctionValue;

use super::CodeGeneration;
use crate::errors::CodeGenerationError;
use crate::function_helpers::function_matches_arity;
use crate::symbols::{MangleConfig, collect_symbol_table};
use x_diagnostics::backend_messages;
use x_mir::MirProgram;

impl<'ctx> CodeGeneration<'ctx> {
    pub fn llvm_function_symbol<'a>(&'a self, source_name: &'a str) -> &'a str {
        self.function_symbols
            .get(source_name)
            .map(String::as_str)
            .unwrap_or(source_name)
    }

    pub(crate) fn has_exact_source_function(&self, source_name: &str) -> bool {
        self.function_symbols.contains_key(source_name)
    }

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

    pub fn llvm_global_symbol<'a>(&'a self, source_name: &'a str) -> &'a str {
        self.global_symbols
            .get(source_name)
            .map(String::as_str)
            .unwrap_or(source_name)
    }

    pub(super) fn prepare_symbol_tables(
        &mut self,
        mir_program: &MirProgram,
    ) -> Result<(), CodeGenerationError> {
        let table = collect_symbol_table(
            mir_program,
            MangleConfig {
                enabled: self.options.mangle_symbols,
                export_global_symbols: self.options.export_global_symbols,
            },
        );
        let mut identities = collections::HashMap::new();

        for symbol in &table.symbols {
            if let Some(previous) = identities.insert(&symbol.symbol_name, symbol)
                && (previous.source_name != symbol.source_name || previous.kind != symbol.kind)
            {
                return Err(CodeGenerationError::InvalidMir(
                    backend_messages::llvm_symbol_identity_collision(
                        &symbol.symbol_name,
                        &previous.source_name,
                        &symbol.source_name,
                    ),
                ));
            }
        }
        self.function_symbols = table.function_map().into_iter().collect();
        self.global_symbols = table.static_map().into_iter().collect();

        Ok(())
    }
}
