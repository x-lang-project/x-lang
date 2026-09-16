use inkwell::values::BasicValueEnum;
use inkwell::{AddressSpace, targets, values};

use super::FunctionCompiler;
use crate::{codegen, errors::CodeGenerationError};
use x_ast::{ReferenceQualifiers, Type};
use x_diagnostics::backend_messages;
use x_mir::{Operand, Place};

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn constrain_safe_reference(
        &self,
        pointer: values::PointerValue<'ctx>,
        place: &Place,
        qualifiers: ReferenceQualifiers,
    ) -> Result<values::PointerValue<'ctx>, CodeGenerationError> {
        let ReferenceQualifiers { is_mut, .. } = qualifiers;

        if self.codegen.pointer_address_space()
            != AddressSpace::from(codegen::CHERI_CAPABILITY_ADDRESS_SPACE)
        {
            return Ok(pointer);
        }

        let place_type = self.get_place_type(place);
        let llvm_type = self.codegen.resolve_llvm_type(&place_type)?;
        let data_layout = self.m().get_data_layout();
        let layout_text = data_layout.as_str().to_str().map_err(|_| {
            CodeGenerationError::LlvmInternalError(
                backend_messages::cheri_target_data_layout_is_not_valid_utf(),
            )
        })?;
        let target_data = targets::TargetData::create(layout_text);
        let size = target_data.get_store_size(&llvm_type);

        if size == 0 {
            return Err(CodeGenerationError::LlvmInternalError(
                backend_messages::cannot_create_a_bounded_safe_reference_to_a_zero_sized_object(),
            ));
        }

        let (path, root_type, offset) = self.cheri_projection(place, &target_data)?;
        let root_llvm_type = self.codegen.resolve_llvm_type(&root_type)?;
        let root_size = target_data.get_store_size(&root_llvm_type);

        self.codegen
            .record_cheri_projection(codegen::CheriProjectionRecord {
                function: sanitize_trace_field(self.function_name),
                path: sanitize_trace_field(&path),
                root_size,
                offset,
                referent_size: size,
                permission_class: codegen::CheriPermissionClass::from_qualifiers(qualifiers),
            });

        let bounded = self.set_exact_reference_bounds(pointer, size)?;

        if is_mut {
            return Ok(bounded);
        }

        self.restrict_reference_store_permissions(bounded)
    }

    fn set_exact_reference_bounds(
        &self,
        pointer: values::PointerValue<'ctx>,
        size: u64,
    ) -> Result<values::PointerValue<'ctx>, CodeGenerationError> {
        let pointer_type = self.pointer_type();
        let i64_type = self.i64();
        let bounds_type = pointer_type.fn_type(&[pointer_type.into(), i64_type.into()], false);
        let bounds = self
            .codegen
            .ensure_external_function("__x_cheri_bounds_set_exact", bounds_type);

        self.call(
            bounds,
            &[pointer.into(), i64_type.const_int(size, false).into()],
            "cheri_bounds_exact",
        )?
        .try_as_basic_value()
        .basic()
        .ok_or_else(|| {
            CodeGenerationError::LlvmInternalError(
                backend_messages::cheri_bounds_intrinsic_returned_no_capability(),
            )
        })
        .map(BasicValueEnum::into_pointer_value)
    }

    fn restrict_reference_store_permissions(
        &self,
        bounded: values::PointerValue<'ctx>,
    ) -> Result<values::PointerValue<'ctx>, CodeGenerationError> {
        let pointer_type = self.pointer_type();
        let i64_type = self.i64();
        let permissions_type = pointer_type.fn_type(&[pointer_type.into(), i64_type.into()], false);
        let permissions = self
            .codegen
            .ensure_external_function("__x_cheri_perms_and", permissions_type);
        // Clear Permit_Store, Permit_Store_Capability, and
        // Permit_Store_Local_Capability while preserving all other authority.
        let readonly_mask = !0x68_u64;

        self.call(
            permissions,
            &[
                bounded.into(),
                i64_type.const_int(readonly_mask, false).into(),
            ],
            "cheri_readonly",
        )?
        .try_as_basic_value()
        .basic()
        .ok_or_else(|| {
            CodeGenerationError::LlvmInternalError(
                backend_messages::cheri_permission_intrinsic_returned_no_capability(),
            )
        })
        .map(BasicValueEnum::into_pointer_value)
    }

    fn cheri_projection(
        &self,
        place: &Place,
        target_data: &targets::TargetData,
    ) -> Result<(String, Type, Option<u64>), CodeGenerationError> {
        match place {
            Place::Local(identifier) => {
                let name = self
                    .body
                    .locals
                    .get(identifier)
                    .and_then(|declaration| declaration.name.clone())
                    .unwrap_or_else(|| format!("_{}", identifier.0));

                Ok((name, self.get_place_type(place), Some(0)))
            }
            Place::Global { name, value_type } => {
                Ok((format!("global::{name}"), value_type.clone(), Some(0)))
            }
            Place::Dereference { base, .. } => {
                let (path, _, _) = self.cheri_projection(base, target_data)?;

                Ok((format!("*{path}"), self.get_place_type(place), Some(0)))
            }
            Place::Index { base, index, .. } => {
                let (base_path, root_type, base_offset) =
                    self.cheri_projection(base, target_data)?;
                let index_value = match index {
                    Operand::ConstantInteger(value) if *value >= 0 => Some(*value as u64),
                    _ => None,
                };

                let element_type = self.get_place_type(place);
                let element_llvm_type = self.codegen.resolve_llvm_type(&element_type)?;
                let element_size = target_data.get_store_size(&element_llvm_type);
                let offset = base_offset
                    .zip(index_value)
                    .and_then(|(base, index)| index.checked_mul(element_size)?.checked_add(base));
                let index_text = index_value
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "?".to_string());

                Ok((format!("{base_path}[{index_text}]"), root_type, offset))
            }
            Place::Field {
                base, field_name, ..
            } => {
                let (base_path, root_type, base_offset) =
                    self.cheri_projection(base, target_data)?;
                let mut base_type = self.get_place_type(base);

                while let Type::Reference { inner, .. } | Type::RawPointer { inner, .. } = base_type
                {
                    base_type = *inner;
                }

                let field_index = match &base_type {
                    Type::Custom(name) | Type::GenericInstance { name, .. } => self
                        .codegen
                        .records
                        .get(name)
                        .and_then(|record| record.field_index(field_name)),
                    Type::Tuple(_) => field_name.parse::<u32>().ok(),
                    _ => None,
                };

                let field_offset = field_index.and_then(|index| {
                    let llvm_type = self.codegen.resolve_llvm_type(&base_type).ok()?;

                    if !llvm_type.is_struct_type() {
                        return None;
                    }

                    target_data.offset_of_element(&llvm_type.into_struct_type(), index)
                });
                let offset = base_offset
                    .zip(field_offset)
                    .and_then(|(base, field)| base.checked_add(field));

                Ok((format!("{base_path}.{field_name}"), root_type, offset))
            }
        }
    }
}

fn sanitize_trace_field(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], "_")
}
