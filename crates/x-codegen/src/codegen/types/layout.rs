use inkwell::targets::TargetData;
use inkwell::types::BasicType;

use super::super::CodeGeneration;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;

impl CodeGeneration<'_> {
    pub(crate) fn target_data(&self) -> TargetData {
        TargetData::create(&self.module.get_data_layout().as_str().to_string_lossy())
    }
}
