use inkwell::module::Linkage;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;

use super::CodeGeneration;
use crate::codegen::items::{MirProgramItems, StaticItem};
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::{MirProgram, Operand};

impl<'ctx> CodeGeneration<'ctx> {
    pub(super) fn declare_static_storage_items(
        &self,
        mir_program: &MirProgram,
    ) -> Result<(), CodeGenerationError> {
        for item in mir_program.statics() {
            self.declare_static_storage(item)?;
        }

        Ok(())
    }

    pub(super) fn declare_static_storage(
        &self,
        item: StaticItem<'_>,
    ) -> Result<(), CodeGenerationError> {
        let StaticItem {
            name,
            value_type,
            initializer,
            is_constant,
        } = item;
        let llvm_name = self.llvm_global_symbol(name);

        if self.module.get_global(llvm_name).is_some() {
            return Ok(());
        }

        if matches!(value_type, Type::Array(_)) {
            return Err(CodeGenerationError::LlvmInternalError(backend_messages::global_name_has_unsized_array_type_fixed_storage_must_be_lowered_as(name)));
        }

        let llvm_type = self.resolve_llvm_type(value_type)?;
        let global =
            self.module
                .add_global(llvm_type, Some(self.pointer_address_space()), llvm_name);
        let linkage = if self.options.export_global_symbols {
            Linkage::External
        } else {
            Linkage::Internal
        };

        global.set_linkage(linkage);
        global.set_constant(is_constant);
        if let Some(value) = self.static_initializer_value(value_type, initializer, llvm_type)? {
            global.set_initializer(&value);
        } else {
            global.set_initializer(&llvm_type.const_zero());
        }

        Ok(())
    }

    pub(super) fn static_initializer_value(
        &self,
        value_type: &Type,
        initializer: Option<&Operand>,
        llvm_type: BasicTypeEnum<'ctx>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenerationError> {
        let Some(initializer) = initializer else {
            return Ok(None);
        };

        let value: BasicValueEnum<'ctx> = match (value_type, initializer) {
            (Type::F32, Operand::ConstantFloat(value)) => {
                self.context.f32_type().const_float(*value).into()
            }
            (Type::F64, Operand::ConstantFloat(value)) => {
                self.context.f64_type().const_float(*value).into()
            }
            _ => return Err(CodeGenerationError::UnsupportedType(value_type.clone())),
        };

        if value.get_type() == llvm_type {
            Ok(Some(value))
        } else {
            Err(CodeGenerationError::UnsupportedType(value_type.clone()))
        }
    }
}
