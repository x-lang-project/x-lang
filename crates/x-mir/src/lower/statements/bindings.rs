use super::MirLowerer;
use crate::errors::MirLowerError;
use crate::lower::program::infer_tuple_pattern_value_type;
use crate::lower::{canonical_array_expression_type, resolved_expression_type};
use crate::{MirStatement, Operand, Place, Rvalue};
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::mir_messages;

impl MirLowerer {
    pub(super) fn lower_identifier_binding(
        &mut self,
        name: String,
        type_annotation: Option<Type>,
        value: Expression,
    ) -> Result<(), MirLowerError> {
        let is_uninitialized_declaration =
            matches!(value.kind, ExpressionKind::TypeLiteral(_)) && type_annotation.is_some();
        let variable_type = match &value.kind {
            ExpressionKind::StructInstantiate(_) => type_annotation
                .clone()
                .unwrap_or_else(|| value.value_type.clone()),
            ExpressionKind::Array(items) => type_annotation
                .clone()
                .unwrap_or_else(|| canonical_array_expression_type(&value.value_type, items.len())),
            ExpressionKind::TypeLiteral(_) => {
                type_annotation.clone().unwrap_or(value.value_type.clone())
            }
            _ => resolved_expression_type(&value).unwrap_or_else(|| value.value_type.clone()),
        };

        let local_identifier = self.allocate_local(Some(name.clone()), variable_type.clone());
        self.variable_mappings.insert(name, local_identifier);

        self.emit_statement(MirStatement::StorageLive(local_identifier))?;

        if is_uninitialized_declaration {
            return Ok(());
        }

        if matches!(value.kind, ExpressionKind::If { .. }) {
            self.lower_if_expression_into_place(value, local_identifier, variable_type)?;

            return Ok(());
        }

        let finalized_operator = self.lower_expression_operand(value, Some(&variable_type))?;

        if self.current_block.is_none() {
            return Ok(());
        }

        self.emit_statement(MirStatement::Assign(
            Place::Local(local_identifier),
            Rvalue::Use(finalized_operator),
        ))?;

        Ok(())
    }

    pub(super) fn lower_tuple_binding(
        &mut self,
        names: Vec<String>,
        value: Expression,
    ) -> Result<(), MirLowerError> {
        let tuple_type = infer_tuple_pattern_value_type(&value, names.len())?;
        let element_types = match &tuple_type {
            Type::Tuple(items) => items.clone(),
            _ => {
                return Err(MirLowerError::UnsupportedExpression(
                    mir_messages::tuple_binding_layout(),
                ));
            }
        };

        let tuple_temporary = self.allocate_local(None, tuple_type);
        let finalized_operator = self.lower_buffered_expression(value, None)?;

        self.emit_statement(MirStatement::StorageLive(tuple_temporary))?;
        self.emit_statement(MirStatement::Assign(
            Place::Local(tuple_temporary),
            Rvalue::Use(finalized_operator),
        ))?;

        for (index, (name, element_type)) in names.into_iter().zip(element_types).enumerate() {
            if name == "_" {
                continue;
            }

            let local_identifier = self.allocate_local(Some(name.clone()), element_type.clone());
            self.variable_mappings.insert(name, local_identifier);
            self.emit_statement(MirStatement::StorageLive(local_identifier))?;
            self.emit_statement(MirStatement::Assign(
                Place::Local(local_identifier),
                Rvalue::Use(Operand::Copy(Box::new(Place::Field {
                    base: Box::new(Place::Local(tuple_temporary)),
                    field_name: index.to_string(),
                    value_type: element_type,
                }))),
            ))?;
        }

        Ok(())
    }
}
