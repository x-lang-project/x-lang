use super::{FunctionSignature, SymbolTable};
use x_ast::Type;
use x_diagnostics::frontend_messages;

impl SymbolTable {
    pub fn struct_fields(&self, name: &str) -> Option<&Vec<(String, Type)>> {
        self.records.fields().get(name)
    }

    pub(crate) fn vacant_generated_name(
        &self,
        base: &str,
    ) -> Result<String, frontend_messages::FrontendIssue> {
        let occupied = |name: &str| {
            self.reserved_source_names.contains(name)
                || self.functions.contains_key(name)
                || self.records.fields().contains_key(name)
                || self.generic_functions.contains_key(name)
                || self.records.templates().contains_key(name)
                || self.variables.iter().any(|scope| scope.contains_key(name))
        };

        let mut candidate = base.to_string();
        let mut suffix: usize = 0;

        while occupied(&candidate) {
            suffix = suffix
                .checked_add(1)
                .ok_or(frontend_messages::FrontendIssue::GeneratedSymbolSpaceExhausted)?;
            candidate = format!("{base}__{suffix}");
        }

        Ok(candidate)
    }

    pub fn add_function(&mut self, name: String, signature: FunctionSignature) {
        match self.qualified_function_cache.get_mut() {
            Ok(cache) => cache.clear(),
            Err(poisoned) => poisoned.into_inner().clear(),
        }

        self.functions.insert(name, signature);
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionSignature> {
        self.functions.get(name)
    }

    pub fn add_struct_method(
        &mut self,
        struct_name: String,
        method_name: String,
        signature: FunctionSignature,
    ) {
        self.struct_methods
            .entry(struct_name)
            .or_default()
            .insert(method_name, signature);
    }

    pub fn get_struct_method(
        &self,
        struct_name: &str,
        method_name: &str,
    ) -> Option<&FunctionSignature> {
        self.struct_methods.get(struct_name)?.get(method_name)
    }
}
