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

    pub fn resolve_named_type_size(&self, name: &str) -> Result<u64, CodeGenerationError> {
        self.resolve_named_layout(name).map(|(size, _)| size)
    }

    pub fn resolve_named_type_align(&self, name: &str) -> Result<u64, CodeGenerationError> {
        self.resolve_named_layout(name).map(|(_, align)| align)
    }

    pub(crate) fn resolve_named_layout(
        &self,
        name: &str,
    ) -> Result<(u64, u64), CodeGenerationError> {
        if let Some(value_type) = Type::from_primitive_name(name) {
            return self.resolve_type_layout(&value_type);
        }

        if let Some(value_type) = self.resolve_struct_type(name) {
            if value_type.is_opaque() {
                return Err(CodeGenerationError::TypeNotFound(name.into()));
            }

            let target = self.target_data();

            return Ok((
                target.get_abi_size(&value_type),
                u64::from(target.get_abi_alignment(&value_type)),
            ));
        }

        self.resolve_type_layout(&Type::Custom(name.into()))
    }
}
