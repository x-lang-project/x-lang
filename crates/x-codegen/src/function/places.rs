mod indexing;
mod operands;
mod soa_values;

use inkwell::values::PointerValue;

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::Place;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn resolve_place_pointer(
        &self,
        place: &Place,
    ) -> Result<PointerValue<'ctx>, CodeGenerationError> {
        match place {
            Place::Local(identifier) => self
                .frame
                .pointer(*identifier)
                .ok_or(CodeGenerationError::MissingLocalAllocation(identifier.0)),
            Place::Global { name, .. } => {
                let llvm_name = self.codegen.llvm_global_symbol(name);

                self.m()
                    .get_global(llvm_name)
                    .map(|global| global.as_pointer_value())
                    .ok_or_else(|| {
                        CodeGenerationError::InvalidMir(
                            backend_messages::global_name_has_no_registered_declaration(name),
                        )
                    })
            }
            Place::Field {
                base, field_name, ..
            } => self.resolve_field_pointer(base, field_name),
            Place::Dereference { base, .. } => {
                let base_pointer = self.resolve_place_pointer(base)?;

                if matches!(
                    self.get_place_type(base),
                    Type::RawPointer { .. } | Type::Reference { .. }
                ) {
                    let loaded_address =
                        self.load(self.pointer_type().into(), base_pointer, "deref_addr_load")?;

                    return Ok(loaded_address.into_pointer_value());
                }

                Err(CodeGenerationError::UnsupportedType(
                    self.get_place_type(base),
                ))
            }
            Place::Index { base, index, .. } => self.resolve_index_pointer(base, index),
        }
    }

    fn resolve_field_pointer(
        &self,
        base: &Place,
        field_name: &str,
    ) -> Result<PointerValue<'ctx>, CodeGenerationError> {
        // SoA projects items[i].field as items.field[i].
        if let Some(pointer) = self.resolve_soa_field_pointer(base, field_name)? {
            return Ok(pointer);
        }

        let base_type = self.get_place_type(base);
        let mut base_pointer = self.resolve_place_pointer(base)?;
        let mut aggregate_ast_type = base_type.clone();

        while let Type::Reference { inner, .. } | Type::RawPointer { inner, .. } =
            aggregate_ast_type
        {
            base_pointer = self
                .load(self.pointer_type().into(), base_pointer, "field_ref_base")?
                .into_pointer_value();
            aggregate_ast_type = *inner;
        }

        let field_index = match field_name.parse::<u32>() {
            Ok(index) => index,
            Err(_) => {
                let struct_name = match &aggregate_ast_type {
                    Type::Custom(struct_name) => struct_name.as_str(),
                    Type::GenericInstance { name, .. } => name.as_str(),
                    _ => {
                        return Err(CodeGenerationError::LlvmInternalError(backend_messages::named_field_field_name_requires_a_concrete_struct_base_found_base_type(field_name, base_type)));
                    }
                };

                let canonical_struct_name = self
                    .codegen
                    .resolve_struct_name(struct_name)
                    .map(str::to_string)
                    .unwrap_or_else(|| struct_name.to_string());
                aggregate_ast_type = Type::Custom(canonical_struct_name.clone());

                self.codegen
                    .records
                    .get(&canonical_struct_name)
                    .and_then(|record| record.field_index(field_name))
                    .ok_or_else(|| {
                        CodeGenerationError::LlvmInternalError(backend_messages::field_field_name_not_found_on_struct_canonical_struct_name(field_name, canonical_struct_name))
                    })?
            }
        };

        let aggregate_type = self.codegen.resolve_llvm_type(&aggregate_ast_type)?;

        if !aggregate_type.is_struct_type() {
            return Err(CodeGenerationError::LlvmInternalError(backend_messages::cannot_project_field_field_name_from_non_struct_type_aggregate_ast_type(field_name, aggregate_ast_type)));
        }

        let struct_type = aggregate_type.into_struct_type();

        self.b()
            .build_struct_gep(struct_type, base_pointer, field_index, "field_gep")
            .map_err(Into::into)
    }

    fn resolve_soa_field_pointer(
        &self,
        base: &Place,
        field_name: &str,
    ) -> Result<Option<PointerValue<'ctx>>, CodeGenerationError> {
        if let Place::Index {
            base: array_base,
            index,
            ..
        } = base
        {
            let mut array_type = self.get_place_type(array_base);
            let mut storage_pointer = self.resolve_place_pointer(array_base)?;

            while let Type::Reference { inner, .. } = array_type {
                storage_pointer = self
                    .load(
                        self.pointer_type().into(),
                        storage_pointer,
                        "soa_array_ref_load",
                    )?
                    .into_pointer_value();
                array_type = *inner;
            }

            if let Type::FixedSizedArray(element_type, length) = &array_type
                && let Some(fields) = self.codegen.soa_fields(element_type)
            {
                let field_index = fields
                            .iter()
                            .position(|(name, _)| name == field_name)
                            .ok_or_else(|| {
                                CodeGenerationError::LlvmInternalError(backend_messages::field_field_name_is_not_present_in_soa_element_element_type(field_name, element_type))
                            })?;
                let index_value = self
                    .load_operand_value_with_type(index, &Type::USize)?
                    .into_int_value();
                let bound = index_value.get_type().const_int(*length as u64, false);
                self.check_index_bounds(
                    index_value,
                    bound,
                    "soa_index_in_bounds",
                    "soa_array_bounds",
                )?;

                let storage_type = self
                    .codegen
                    .resolve_llvm_type(&array_type)?
                    .into_struct_type();
                let field_array_pointer = self.struct_gep(
                    storage_type,
                    storage_pointer,
                    field_index as u32,
                    "soa_field_array_gep",
                )?;
                let field_array_type = storage_type
                    .get_field_type_at_index(field_index as u32)
                    .ok_or_else(|| {
                        CodeGenerationError::LlvmInternalError(
                            backend_messages::soa_storage_is_missing_field_array_field_index(
                                field_index,
                            ),
                        )
                    })?
                    .into_array_type();
                let zero = index_value.get_type().const_zero();

                return unsafe {
                    self.b()
                        .build_gep(
                            field_array_type,
                            field_array_pointer,
                            &[zero, index_value],
                            "soa_field_element_gep",
                        )
                        .map_err(Into::into)
                }
                .map(Some);
            }
        }

        Ok(None)
    }
}
