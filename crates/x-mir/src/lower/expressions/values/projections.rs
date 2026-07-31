use super::*;

impl MirLowerer {
    pub(super) fn lower_evaluated_projection(
        &mut self,
        kind: ExpressionKind,
        result_type: &Type,
    ) -> Result<Option<Place>, MirLowerError> {
        let place = match kind {
            ExpressionKind::FieldAccess { object, field } => {
                let value_type = object.value_type.clone();
                let operand = self.lower_evaluated_operand(*object, Some(&value_type))?;
                let base = self.ensure_operand_local(operand, value_type)?;

                Place::Field {
                    base: Box::new(Place::Local(base)),
                    field_name: field,
                    value_type: result_type.clone(),
                }
            }
            ExpressionKind::ArrayAccess { array, index } => {
                let value_type = array.value_type.clone();
                let operand = self.lower_evaluated_operand(*array, Some(&value_type))?;

                if self.current_block.is_none() {
                    return Ok(None);
                }

                let base = self.ensure_operand_local(operand, value_type)?;
                let index = self.lower_evaluated_operand(*index, Some(&Type::USize))?;

                Place::Index {
                    base: Box::new(Place::Local(base)),
                    index,
                    value_type: result_type.clone(),
                }
            }
            ExpressionKind::Dereference { expression } => {
                let value_type = expression.value_type.clone();
                let operand = self.lower_evaluated_operand(*expression, Some(&value_type))?;
                let base = self.ensure_operand_local(operand, value_type)?;

                Place::Dereference {
                    base: Box::new(Place::Local(base)),
                    value_type: result_type.clone(),
                }
            }
            shape => return Err(unsupported_shape(shape)),
        };

        Ok(Some(place))
    }
}
