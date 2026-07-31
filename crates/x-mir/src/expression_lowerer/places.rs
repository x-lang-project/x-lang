use super::ExpressionLowerer;
use crate::errors::MirLowerError;
use crate::{Operand, Place};
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::backend_messages;

impl ExpressionLowerer<'_> {
    pub(super) fn resolve_expression_to_place(
        &mut self,
        expression: Expression,
    ) -> Result<Place, MirLowerError> {
        match expression.kind {
            ExpressionKind::Identifier(name) => self.resolve_identifier_place(name),
            ExpressionKind::Dereference { expression: inner } => {
                self.resolve_deref_place(*inner, expression.value_type)
            }
            ExpressionKind::ArrayAccess { array, index } => {
                self.resolve_array_index_place(*array, *index, expression.value_type)
            }
            ExpressionKind::FieldAccess { object, field } => {
                self.resolve_field_place(*object, field, expression.value_type)
            }
            other => Err(MirLowerError::InvalidMemoryPlace(
                backend_messages::expression_is_not_addressable_other(other),
            )),
        }
    }

    pub(super) fn resolve_identifier_place(&self, name: String) -> Result<Place, MirLowerError> {
        if let Some(local_identifier) = self.variable_mappings.get(&name).copied() {
            Ok(Place::Local(local_identifier))
        } else if let Some(global_type) = self.global_mappings.get(&name).cloned() {
            Ok(Place::Global {
                name,
                value_type: global_type,
            })
        } else if let Some((global_name, global_type)) = self.resolve_unique_global_suffix(&name) {
            Ok(Place::Global {
                name: global_name,
                value_type: global_type,
            })
        } else {
            Err(MirLowerError::UnallocatedVariable(name))
        }
    }

    pub(super) fn resolve_unique_global_suffix(&self, name: &str) -> Option<(String, Type)> {
        let suffix = format!("::{name}");
        let mut matches = self
            .global_mappings
            .iter()
            .filter(|(candidate, _)| candidate.ends_with(&suffix));
        let (candidate, value_type) = matches.next()?;

        matches
            .next()
            .is_none()
            .then(|| (candidate.clone(), value_type.clone()))
    }

    fn resolve_deref_place(
        &mut self,
        inner: Expression,
        fallback_type: Type,
    ) -> Result<Place, MirLowerError> {
        let inner_type = inner.value_type.clone();
        let inner_operator = self.lower_expression_with_context(inner, None)?;
        let resolved_type = self.dereference_type(&inner_operator, &inner_type, fallback_type);

        match inner_operator {
            Operand::Copy(place) | Operand::Move(place) => Ok(Place::Dereference {
                base: place,
                value_type: resolved_type,
            }),
            other => Err(MirLowerError::InvalidMemoryPlace(
                backend_messages::deref_source_is_not_a_resolvable_place_operand_other(other),
            )),
        }
    }

    fn resolve_array_index_place(
        &mut self,
        array: Expression,
        index: Expression,
        value_type: Type,
    ) -> Result<Place, MirLowerError> {
        let array_place = self.lower_expression_to_place(array, None, "array base")?;
        let index = self.lower_expression_with_context(index, Some(&Type::USize))?;

        Ok(Place::Index {
            base: Box::new(array_place),
            index,
            value_type,
        })
    }

    fn resolve_field_place(
        &mut self,
        object: Expression,
        field: String,
        value_type: Type,
    ) -> Result<Place, MirLowerError> {
        let object = self.lower_expression_to_place(object, None, "field object")?;

        Ok(Place::Field {
            base: Box::new(object),
            field_name: field,
            value_type,
        })
    }

    fn lower_expression_to_place(
        &mut self,
        expression: Expression,
        expected_type: Option<&Type>,
        context: &str,
    ) -> Result<Place, MirLowerError> {
        match self.lower_expression_with_context(expression, expected_type)? {
            Operand::Copy(place) | Operand::Move(place) => Ok(*place),
            other => Err(MirLowerError::InvalidMemoryPlace(
                backend_messages::context_is_not_a_resolvable_place_operand_other(context, other),
            )),
        }
    }
}
