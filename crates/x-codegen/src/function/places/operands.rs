use std::iter;

use inkwell::module;
use inkwell::values::BasicValueEnum;

use super::super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::{Operand, Place};

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(in crate::function) fn load_operand_value(
        &self,
        operand: &Operand,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        match operand {
            &Operand::ConstantFloat(value) => Ok(self.context().f64_type().const_float(value).into()),
            &Operand::ConstantInteger(value) => {
                let value_type = self.codegen.resolve_llvm_type(&self.body.operand_type(operand))?;

                Ok(value_type.into_int_type().const_int(value as u64, true).into())
            }
            &Operand::ConstantBoolean(value) => Ok(self
                .context()
                .bool_type()
                .const_int(if value { 1 } else { 0 }, false)
                .into()),
            &Operand::ConstantCharacter(value) => {
                let value_type = self.codegen.resolve_llvm_type(&Type::Character)?;

                Ok(value_type.into_int_type().const_int(value as u64, false).into())
            }
            Operand::ConstantString(value) => {
                // Build bytes directly to preserve embedded NULs.
                let bytes = value.bytes().chain(iter::once(0))
                    .map(|byte| self.context().i8_type().const_int(u64::from(byte), false))
                    .collect::<Vec<_>>();
                let initializer = self.context().i8_type().const_array(&bytes);
                let storage = self.m().add_global(initializer.get_type(),
                    Some(self.codegen.pointer_address_space()), "x_const_str");
                storage.set_initializer(&initializer);
                storage.set_constant(true);
                storage.set_linkage(module::Linkage::Private);
                let data = storage.as_pointer_value();

                let descriptor_type = self.pointer_length_type();
                let descriptor = self.m().add_global(
                    descriptor_type,
                    Some(self.codegen.pointer_address_space()),
                    "x_const_str_desc",
                );
                descriptor.set_linkage(module::Linkage::Private);
                let initializer = descriptor_type.const_named_struct(&[
                    data.const_cast(self.pointer_type()).into(),
                    self.usize_type().const_int(value.len() as u64, false).into(),
                ]);
                descriptor.set_initializer(&initializer);
                descriptor.set_constant(true);

                Ok(descriptor.as_pointer_value().into())
            }
            Operand::Copy(place) | Operand::Move(place) => {
                if let Some(value) = self.load_soa_element(place)? { return Ok(value); }

                let pointer = self.resolve_place_pointer(place)?;

                let element_type = match &**place {
                    Place::Global { value_type, .. } => self.codegen.resolve_llvm_type(value_type)?,
                    Place::Local(_) | Place::Index { .. } | Place::Field { .. } => self
                        .codegen
                        .resolve_llvm_type(&self.get_place_type(place))?,
                    Place::Dereference { .. } => {
                        let deref_type = self.get_place_type(place);

                        self.codegen.resolve_llvm_type(&deref_type)?
                    }
                };

                self.load(element_type , pointer, "operand_load_tmp")
            }
            other => Err(CodeGenerationError::LlvmInternalError(backend_messages::unresolved_operand_context_match_fallback_execution_path_error_other(other))),
        }
    }

    pub(in crate::function) fn load_operand_value_with_type(
        &self,
        operand: &Operand,
        expected_type: &Type,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        match operand {
            &Operand::ConstantInteger(value) => {
                let llvm_type = self.codegen.resolve_llvm_type(expected_type)?;

                if llvm_type.is_int_type() {
                    Ok(llvm_type
                        .into_int_type()
                        .const_int(value as u64, true)
                        .into())
                } else if llvm_type.is_float_type() {
                    Ok(llvm_type.into_float_type().const_float(value as f64).into())
                } else {
                    Err(CodeGenerationError::UnsupportedType(expected_type.clone()))
                }
            }
            Operand::Copy(place) | Operand::Move(place) => {
                if let Some(value) = self.load_soa_element(place)? {
                    let value_type = self.get_place_type(place);

                    return self.normalize_value_for_store(
                        crate::function::numeric::TypedValue {
                            value,
                            value_type: &value_type,
                        },
                        expected_type,
                        self.codegen.resolve_llvm_type(expected_type)?,
                        "soa_gather_adjust",
                    );
                }

                let pointer = self.resolve_place_pointer(place)?;
                let expected_llvm_type = self.codegen.resolve_llvm_type(expected_type)?;
                let place_type = self.get_place_type(place);
                let load_type = if matches!(place_type, Type::Unknown | Type::Infer(_)) {
                    expected_llvm_type
                } else {
                    self.codegen.resolve_llvm_type(&place_type)?
                };

                let loaded = self.load(load_type, pointer, "operand_typed_load")?;

                self.normalize_value_for_store(
                    crate::function::numeric::TypedValue {
                        value: loaded,
                        value_type: &place_type,
                    },
                    expected_type,
                    expected_llvm_type,
                    "operand_typed_adjust",
                )
            }
            _ => self.load_operand_value(operand),
        }
    }
}
