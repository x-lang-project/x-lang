use super::*;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(in crate::function) fn struct_initialiser_field_index(
        &self,
        name: &str,
        field: &str,
    ) -> Result<u32, CodeGenerationError> {
        self.codegen
            .resolve_struct_name(name)
            .and_then(|name| self.codegen.records.get(name))
            .and_then(|record| record.field_index(field))
            .ok_or_else(|| {
                CodeGenerationError::InvalidMir(
                    backend_messages::unknown_struct_initializer_field_name_field(name, field),
                )
            })
    }

    pub(in crate::function) fn compile_struct_rvalue(
        &self,
        name: &str,
        fields: &[(String, Operand)],
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let Some(struct_type) = self.codegen.resolve_struct_type(name) else {
            return Err(CodeGenerationError::LlvmInternalError(
                backend_messages::cannot_instantiate_unresolved_struct_name(name),
            ));
        };

        if fields.len() != struct_type.count_fields() as usize {
            return Err(CodeGenerationError::InvalidMir(
                backend_messages::struct_initializer_requires_all_declared_fields(name),
            ));
        }

        let mut initialized = std::collections::HashSet::new();

        for (field, _) in fields {
            let index = self.struct_initialiser_field_index(name, field)?;

            if !initialized.insert(index) {
                return Err(CodeGenerationError::InvalidMir(
                    backend_messages::duplicate_struct_initializer_field_name_field_name(
                        name, field,
                    ),
                ));
            }
        }

        let field_types = struct_type.get_field_types();

        let mut aggregate = struct_type.get_undef();

        for (field_name, operand) in fields {
            let index = self.struct_initialiser_field_index(name, field_name)?;
            let mut field_value = self.load_operand_value(operand)?;

            if let Some(expected_field_type) = field_types.get(index as usize).copied() {
                let operand_type = self.operand_ast_type(operand).unwrap_or(Type::Unknown);
                field_value = self.normalize_value_for_store(
                    crate::function::numeric::TypedValue {
                        value: field_value,
                        value_type: &operand_type,
                    },
                    &self
                        .codegen
                        .resolve_struct_field_type(name, field_name)
                        .ok_or(CodeGenerationError::UnsupportedType(Type::Unknown))?,
                    expected_field_type,
                    "struct_field_type_adjust",
                )?;
            }

            let inserted =
                self.insert_value(aggregate.into(), field_value, index, "struct_field_insert")?;
            aggregate = inserted.into_struct_value();
        }

        Ok(aggregate.into())
    }
}
