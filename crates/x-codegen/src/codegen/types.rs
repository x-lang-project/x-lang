mod layout;
mod records;

use inkwell::types;
use inkwell::types::{BasicType, BasicTypeEnum};

use super::CodeGeneration;
use crate::errors::CodeGenerationError;
use x_ast::Type;

impl<'ctx> CodeGeneration<'ctx> {
    fn resolve_simd_type(
        &self,
        value_type: &Type,
        element: &Type,
        lanes: usize,
    ) -> Result<BasicTypeEnum<'ctx>, CodeGenerationError> {
        if lanes == 0 {
            return Err(CodeGenerationError::UnsupportedType(value_type.clone()));
        }

        let element_llvm_type = self.resolve_llvm_type(element)?;

        match element_llvm_type {
            BasicTypeEnum::IntType(integer_type) => Ok(integer_type.vec_type(lanes as u32).into()),
            BasicTypeEnum::FloatType(float_type) => Ok(float_type.vec_type(lanes as u32).into()),
            _ => Err(CodeGenerationError::UnsupportedType(value_type.clone())),
        }
    }

    pub(super) fn pointer_type(&self) -> types::PointerType<'ctx> {
        self.context.ptr_type(self.pointer_address_space())
    }

    pub(super) fn primitive_llvm_type(&self, value_type: &Type) -> Option<BasicTypeEnum<'ctx>> {
        Some(match value_type {
            Type::I8 | Type::U8 | Type::Character | Type::Void => self.context.i8_type().into(),
            Type::I16 | Type::U16 => self.context.i16_type().into(),
            Type::I32 | Type::U32 => self.context.i32_type().into(),
            Type::I64 | Type::U64 => self.context.i64_type().into(),
            Type::ISize | Type::USize => self
                .context
                .ptr_sized_int_type(&self.target_data(), None)
                .into(),
            Type::F32 => self.context.f32_type().into(),
            Type::F64 => self.context.f64_type().into(),
            Type::Boolean => self.context.bool_type().into(),
            Type::RawPointer { .. }
            | Type::Reference { .. }
            | Type::Array(_)
            | Type::String
            | Type::Function { .. } => self.pointer_type().into(),
            _ => return None,
        })
    }

    pub fn resolve_llvm_type(
        &self,
        value_type: &Type,
    ) -> Result<BasicTypeEnum<'ctx>, CodeGenerationError> {
        if let Some(llvm_type) = self.primitive_llvm_type(value_type) {
            return Ok(llvm_type);
        }

        match value_type {
            Type::Simd { element, lanes } => self.resolve_simd_type(value_type, element, *lanes),
            Type::Tuple(element_types) => {
                let llvm_fields = element_types
                    .iter()
                    .map(|value_type| self.resolve_llvm_type(value_type))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(self.context.struct_type(&llvm_fields, false).into())
            }
            Type::Custom(struct_name)
            | Type::GenericInstance {
                name: struct_name, ..
            } => {
                if let Some(current_struct_type) = self.resolve_struct_type(struct_name) {
                    Ok(current_struct_type.into())
                } else {
                    Err(CodeGenerationError::UnsupportedType(value_type.clone()))
                }
            }
            _ => Err(CodeGenerationError::UnsupportedType(value_type.clone())),
        }
    }
}
