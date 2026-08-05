use super::{FunctionSignature, SymbolTable};
use x_ast::Type;
use x_diagnostics::frontend_messages;

impl SymbolTable {
    pub fn add_function(&mut self, name: String, signature: FunctionSignature) {
        match self.qualified_function_cache.get_mut() {
            Ok(cache) => cache.clear(),
            Err(poisoned) => poisoned.into_inner().clear(),
        }

        self.functions.insert(name, signature);
    }
}
