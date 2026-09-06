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

    pub(crate) fn resolve_type_layout(
        &self,
        value_type: &Type,
    ) -> Result<(u64, u64), CodeGenerationError> {
        if matches!(
            value_type,
            Type::Void | Type::Unknown | Type::Infer(_) | Type::TypeParameter(_)
        ) {
            return Err(CodeGenerationError::UnsupportedType(value_type.clone()));
        }

        let llvm_type = self.resolve_llvm_type(value_type)?;

        if llvm_type.size_of().is_none() {
            return Err(CodeGenerationError::UnsupportedType(value_type.clone()));
        }

        let target = self.target_data();

        Ok((
            target.get_abi_size(&llvm_type),
            u64::from(target.get_abi_alignment(&llvm_type)),
        ))
    }
}
