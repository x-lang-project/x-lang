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

    pub(crate) fn resolve_named_field_offset(
        &self,
        name: &str,
        field: &str,
    ) -> Result<u64, CodeGenerationError> {
        let canonical = self
            .resolve_struct_name(name)
            .ok_or_else(|| CodeGenerationError::TypeNotFound(name.into()))?;
        let index = self
            .records
            .get(canonical)
            .and_then(|record| record.field_index(field))
            .ok_or_else(|| {
                CodeGenerationError::InvalidMir(
                    backend_messages::unknown_field_field_in_structure_name(field, name),
                )
            })?;
        let structure = self
            .resolve_struct_type(canonical)
            .ok_or_else(|| CodeGenerationError::TypeNotFound(canonical.into()))?;

        if structure.is_opaque() || index >= structure.count_fields() {
            return Err(CodeGenerationError::InvalidMir(
                backend_messages::field_name_field_has_no_concrete_layout(name, field),
            ));
        }

        self.target_data()
            .offset_of_element(&structure, index)
            .ok_or_else(|| {
                CodeGenerationError::InvalidMir(
                    backend_messages::cannot_determine_offset_of_name_field(name, field),
                )
            })
    }
}
