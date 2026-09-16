mod records;
use inkwell::types::{BasicType, StructType};
use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::{Operand, Place};

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn compile_tuple_rvalue(
        &self,
        elements: &[Operand],
        target_place: &Place,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let target_type = self.get_place_type(target_place);
        let Type::Tuple(items) = &target_type else {
            return Err(CodeGenerationError::UnsupportedType(target_type));
        };

        if items.len() != elements.len() {
            return Err(CodeGenerationError::UnsupportedType(target_type));
        }

        let tuple_items = Some(items.as_slice());

        let tuple_type = self.codegen.resolve_llvm_type(&target_type)?;

        let tuple_type = tuple_type.into_struct_type();
        let field_types = tuple_type.get_field_types();
        let mut aggregate = tuple_type.get_undef();

        for (index, operand) in elements.iter().enumerate() {
            let current_operand_ast_type = tuple_items
                .and_then(|items| items.get(index))
                .cloned()
                .or_else(|| self.operand_ast_type(operand))
                .unwrap_or(Type::Unknown);
            let mut value =
                self.load_operand_value_with_type(operand, &current_operand_ast_type)?;

            if let Some(expected_field_type) = field_types.get(index).copied() {
                value = self.normalize_value_for_store(
                    crate::function::numeric::TypedValue {
                        value,
                        value_type: &current_operand_ast_type,
                    },
                    &current_operand_ast_type,
                    expected_field_type,
                    "tuple_field_type_adjust",
                )?;
            }

            let inserted =
                self.insert_value(aggregate.into(), value, index as u32, "tuple_field_insert")?;
            aggregate = inserted.into_struct_value();
        }

        Ok(aggregate.into())
    }

    pub(super) fn compile_array_rvalue(
        &self,
        elements: &[Operand],
        target_place: &Place,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let target_type = self.get_place_type(target_place);

        if let Type::FixedSizedArray(inner, length) = &target_type
            && let Some(fields) = self.codegen.soa_fields(inner)
        {
            if elements.len() != *length {
                return Err(CodeGenerationError::InvalidMir(backend_messages::soa_array_initializer_has_value_elements_but_its_type_requires_length(elements.len(), *length)));
            }

            let storage_type = self
                .codegen
                .resolve_llvm_type(&target_type)?
                .into_struct_type();

            return self.compile_soa_array_rvalue(elements, storage_type, inner, &fields);
        }

        let Type::FixedSizedArray(inner, length) = &target_type else {
            return Err(CodeGenerationError::UnsupportedType(target_type));
        };

        if *length != elements.len() {
            return Err(CodeGenerationError::UnsupportedType(target_type));
        }

        let element_ast_type = Some((**inner).clone());
        let array_type = self
            .codegen
            .resolve_llvm_type(&target_type)?
            .into_array_type();

        let mut aggregate = array_type.get_undef();

        for (index, operand) in elements.iter().enumerate() {
            let mut value = if let Some(element_ast_type) = element_ast_type.as_ref() {
                self.load_operand_value_with_type(operand, element_ast_type)?
            } else {
                self.load_operand_value(operand)?
            };

            let expected_type = array_type.get_element_type();
            let operand_type = element_ast_type
                .as_ref()
                .cloned()
                .or_else(|| self.operand_ast_type(operand))
                .unwrap_or(Type::Unknown);
            value = self.normalize_value_for_store(
                crate::function::numeric::TypedValue {
                    value,
                    value_type: &operand_type,
                },
                &operand_type,
                expected_type,
                "array_elem_type_adjust",
            )?;
            let inserted =
                self.insert_value(aggregate.into(), value, index as u32, "array_elem_insert")?;
            aggregate = inserted.into_array_value();
        }

        Ok(aggregate.into())
    }

    fn compile_soa_array_rvalue(
        &self,
        elements: &[Operand],
        storage_type: StructType<'ctx>,
        inner: &Type,
        fields: &[(String, Type)],
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let mut storage = storage_type.get_undef();

        for (field_index, (_, field_ast_type)) in fields.iter().enumerate() {
            let field_llvm_type = self.codegen.resolve_llvm_type(field_ast_type)?;
            let field_array_type = field_llvm_type.array_type(elements.len() as u32);
            let mut field_array = field_array_type.get_undef();

            for (element_index, operand) in elements.iter().enumerate() {
                let element = self.load_operand_value_with_type(operand, inner)?;

                if !element.is_struct_value() {
                    return Err(CodeGenerationError::InvalidMir(
                        backend_messages::soa_element_element_index_is_not_a_struct_value(
                            element_index,
                        ),
                    ));
                }

                let mut field_value = self.b().build_extract_value(
                    element.into_struct_value(),
                    field_index as u32,
                    "soa_field_extract",
                )?;
                field_value = self.normalize_value_for_store(
                    crate::function::numeric::TypedValue {
                        value: field_value,
                        value_type: field_ast_type,
                    },
                    field_ast_type,
                    field_llvm_type,
                    "soa_field_type_adjust",
                )?;
                field_array = self
                    .insert_value(
                        field_array.into(),
                        field_value,
                        element_index as u32,
                        "soa_element_insert",
                    )?
                    .into_array_value();
            }

            storage = self
                .insert_value(
                    storage.into(),
                    field_array.into(),
                    field_index as u32,
                    "soa_field_array_insert",
                )?
                .into_struct_value();
        }

        Ok(storage.into())
    }
}
