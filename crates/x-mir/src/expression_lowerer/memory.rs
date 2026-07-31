use super::ExpressionLowerer;
use crate::errors::MirLowerError;
use crate::{MirStatement, Operand, Place, Rvalue};
use x_ast::{Expression, ReferenceQualifiers, Type};
use x_diagnostics::backend_messages;

impl ExpressionLowerer<'_> {
    pub(super) fn lower_reference_value(
        &mut self,
        inner_expression: Expression,
        qualifiers: ReferenceQualifiers,
        expected_type: Option<&Type>,
    ) -> Result<Operand, MirLowerError> {
        let ReferenceQualifiers { is_mut, is_unique } = qualifiers;
        let target_type = inner_expression.value_type.clone();
        let temporary_identifier =
            self.allocate_temporary(expected_type.cloned().unwrap_or(Type::Reference {
                lifetime: None,
                is_mut,
                is_unique,
                inner: Box::new(target_type.clone()),
            }));
        let target_place = match self.resolve_expression_to_place(inner_expression.clone()) {
            Ok(place) => place,
            Err(MirLowerError::InvalidMemoryPlace(_)) => {
                let value_operator =
                    self.lower_expression_with_context(inner_expression, Some(&target_type))?;
                let materialized = self.allocate_temporary(target_type);
                self.emit_statement(MirStatement::Assign(
                    Place::Local(materialized),
                    Rvalue::Use(value_operator),
                ));

                Place::Local(materialized)
            }
            Err(error) => return Err(error),
        };

        self.emit_statement(MirStatement::Assign(
            Place::Local(temporary_identifier),
            Rvalue::AddressOf {
                is_mut,
                is_unique,
                expression: target_place,
            },
        ));

        Ok(Operand::Move(Box::new(Place::Local(temporary_identifier))))
    }

    pub(super) fn lower_dereference_value(
        &mut self,
        inner_expression: Expression,
        result_type: Type,
    ) -> Result<Operand, MirLowerError> {
        let inner_type = inner_expression.value_type.clone();
        let inner_operator = self.lower_expression_with_context(inner_expression, None)?;

        let resolved_deref_type = self.dereference_type(&inner_operator, &inner_type, result_type);

        let temporary_identifier = self.allocate_temporary(resolved_deref_type.clone());
        let source_place = match inner_operator {
            Operand::Copy(place) | Operand::Move(place) => *place,
            other => {
                return Err(MirLowerError::InvalidMemoryPlace(
                    backend_messages::deref_source_is_not_a_place_operand_other(other),
                ));
            }
        };

        let deref_place = Place::Dereference {
            base: Box::new(source_place),
            value_type: resolved_deref_type,
        };
        self.emit_statement(MirStatement::Assign(
            Place::Local(temporary_identifier),
            Rvalue::Use(Operand::Copy(Box::new(deref_place))),
        ));

        Ok(Operand::Move(Box::new(Place::Local(temporary_identifier))))
    }

    pub(super) fn lower_index_value(
        &mut self,
        array: Expression,
        index: Expression,
        result_type: Type,
    ) -> Result<Operand, MirLowerError> {
        let array_operator = self.lower_expression_with_context(array, None)?;
        let index_operator = self.lower_expression_with_context(index, Some(&Type::USize))?;
        let array_place = match array_operator {
            Operand::Copy(place) | Operand::Move(place) => *place,
            other => {
                return Err(MirLowerError::InvalidMemoryPlace(
                    backend_messages::array_base_is_not_a_place_operand_other(other),
                ));
            }
        };

        let indexed_place = Place::Index {
            base: Box::new(array_place),
            index: index_operator,
            value_type: result_type.clone(),
        };

        Ok(Operand::Copy(Box::new(indexed_place)))
    }

    pub(super) fn lower_field_value(
        &mut self,
        object: Expression,
        field: String,
        result_type: Type,
    ) -> Result<Operand, MirLowerError> {
        let object_operator = self.lower_expression_with_context(object, None)?;
        let object_place = match object_operator {
            Operand::Copy(place) | Operand::Move(place) => *place,
            other => {
                return Err(MirLowerError::InvalidMemoryPlace(
                    backend_messages::field_object_is_not_a_place_operand_other(other),
                ));
            }
        };

        let field_place = Place::Field {
            base: Box::new(object_place),
            field_name: field,
            value_type: result_type.clone(),
        };

        Ok(Operand::Copy(Box::new(field_place)))
    }

    pub(super) fn lower_assignment_value(
        &mut self,
        target: Expression,
        value: Expression,
    ) -> Result<Operand, MirLowerError> {
        let target_type = target.value_type.clone();
        let value_operator = self.lower_expression_with_context(value, Some(&target_type))?;
        let destination_place = self.resolve_expression_to_place(target)?;

        self.emit_statement(MirStatement::Assign(
            destination_place.clone(),
            Rvalue::Use(value_operator),
        ));

        Ok(Operand::Copy(Box::new(destination_place)))
    }
}
