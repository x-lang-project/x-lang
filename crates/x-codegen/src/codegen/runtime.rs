use inkwell::module::Linkage;
use inkwell::types::FunctionType;
use inkwell::values::FunctionValue;

use super::CodeGeneration;
use x_runtime::abi;

impl<'ctx> CodeGeneration<'ctx> {
    pub(crate) fn ensure_external_function(
        &self,
        name: &str,
        function_type: FunctionType<'ctx>,
    ) -> FunctionValue<'ctx> {
        self.module.get_function(name).unwrap_or_else(|| {
            self.module
                .add_function(name, function_type, Some(Linkage::External))
        })
    }
}
