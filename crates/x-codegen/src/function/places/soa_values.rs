use inkwell::values::{BasicValueEnum, PointerValue};

use super::super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::{LocalIdentifier, Place};

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(in crate::function) fn soa_element_fields(
        &self,
        place: &Place,
    ) -> Option<Vec<(String, Type)>> {
        let Place::Index { base, .. } = place else {
            return None;
        };

        let mut value_type = self.get_place_type(base);

        while let Type::Reference { inner, .. } = value_type {
            value_type = *inner;
        }

        match value_type {
            Type::FixedSizedArray(element, _) => self.codegen.soa_fields(&element),
            _ => None,
        }
    }

    pub(in crate::function) fn soa_element_pointers(
        &self,
        place: &Place,
    ) -> Result<Option<Vec<PointerValue<'ctx>>>, CodeGenerationError> {
        let Some(fields) = self.soa_element_fields(place) else {
            return Ok(None);
        };

        if fields.is_empty() {
            // Empty records still require array bounds checks.
            if let Place::Index { base, index, .. } = place {
                let mut value_type = self.get_place_type(base);

                while let Type::Reference { inner, .. } = value_type {
                    value_type = *inner;
                }

                if let Type::FixedSizedArray(_, length) = value_type {
                    let value = self
                        .load_operand_value_with_type(index, &Type::USize)?
                        .into_int_value();
                    self.check_index_bounds(
                        value,
                        value.get_type().const_int(length as u64, false),
                        "soa_empty_in_bounds",
                        "soa_array_bounds",
                    )?;
                }
            }
        }

        fields
            .iter()
            .map(|(name, _)| {
                self.resolve_soa_field_pointer(place, name)
                    .and_then(|pointer| {
                        pointer.ok_or_else(|| {
                            CodeGenerationError::InvalidMir(
                                backend_messages::soa_field_storage_missing(),
                            )
                        })
                    })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some)
    }

    pub(in crate::function) fn load_soa_element(
        &self,
        place: &Place,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenerationError> {
        let Some(fields) = self.soa_element_fields(place) else {
            return Ok(None);
        };

        let pointers = self.soa_element_pointers(place)?.ok_or_else(|| {
            CodeGenerationError::InvalidMir(backend_messages::soa_logical_element_missing())
        })?;
        let value_type = self
            .codegen
            .resolve_llvm_type(&self.get_place_type(place))?
            .into_struct_type();
        let mut value = value_type.get_undef();

        for (index, ((_, ast_type), pointer)) in fields.iter().zip(pointers).enumerate() {
            let field = self.load(
                self.codegen.resolve_llvm_type(ast_type)?,
                pointer,
                "soa_gather_field",
            )?;
            value = self
                .b()
                .build_insert_value(value, field, index as u32, "soa_gather_value")?
                .into_struct_value();
        }

        Ok(Some(value.into()))
    }

    pub(in crate::function) fn store_soa_element(
        &mut self,
        place: &Place,
        pointers: &[PointerValue<'ctx>],
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), CodeGenerationError> {
        // Snapshot every RHS field before the first store.
        let aggregate = value.into_struct_value();

        for (index, pointer) in pointers.iter().enumerate() {
            let field =
                self.b()
                    .build_extract_value(aggregate, index as u32, "soa_scatter_field")?;
            self.store(*pointer, field)?;
        }
        // Run invariants only after all fields have been stored.
        let value_type = self.get_place_type(place);

        if !self.has_soa_value_invariants(&value_type) {
            return self.emit_enclosing_invariants_after_store(place);
        }

        let mut identifier = LocalIdentifier(
            self.body
                .locals
                .keys()
                .map(|identifier| identifier.0)
                .max()
                .unwrap_or(0)
                + 1,
        );

        while self.frame.pointer(identifier).is_some() {
            identifier.0 += 1;
        }

        let temporary = self.alloca(value.get_type(), "soa_invariant_value")?;
        self.store(temporary, value)?;
        self.frame.insert(identifier, value_type.clone(), temporary);
        self.emit_soa_value_invariants(&Place::Local(identifier), &value_type)?;

        self.emit_enclosing_invariants_after_store(place)
    }

    fn emit_soa_value_invariants(
        &self,
        place: &Place,
        value_type: &Type,
    ) -> Result<(), CodeGenerationError> {
        let name = match value_type {
            Type::Custom(name) | Type::GenericInstance { name, .. } => name,
            _ => return Ok(()),
        };

        let canonical = self.codegen.resolve_struct_name(name).unwrap_or(name);

        if let Some(record) = self.codegen.records.get(canonical) {
            for (field, field_type) in record.fields() {
                if matches!(field_type, Type::Custom(_) | Type::GenericInstance { .. }) {
                    self.emit_soa_value_invariants(
                        &Place::Field {
                            base: Box::new(place.clone()),
                            field_name: field.clone(),
                            value_type: field_type.clone(),
                        },
                        field_type,
                    )?;
                }
            }
        }

        self.emit_struct_invariant(name, place)
    }

    fn has_soa_value_invariants(&self, value_type: &Type) -> bool {
        let name = match value_type {
            Type::Custom(name) | Type::GenericInstance { name, .. } => name,
            _ => return false,
        };

        let canonical = self.codegen.resolve_struct_name(name).unwrap_or(name);

        self.codegen.records.get(canonical).is_some_and(|record| {
            record.invariant().is_some()
                || record
                    .fields()
                    .iter()
                    .any(|(_, field)| self.has_soa_value_invariants(field))
        })
    }
}
