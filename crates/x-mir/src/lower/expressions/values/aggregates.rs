use super::*;

impl MirLowerer {
    pub(super) fn lower_evaluated_aggregate(
        &mut self,
        kind: ExpressionKind,
    ) -> Result<Rvalue, MirLowerError> {
        match kind {
            ExpressionKind::Tuple(items) => Ok(Rvalue::Tuple(self.lower_ordered_items(items)?)),
            ExpressionKind::Array(items) => Ok(Rvalue::Array(self.lower_ordered_items(items)?)),
            ExpressionKind::ArrayRepeat { value, count } => {
                let length = crate::lower::array_repeat_length(&count)?;
                let value_type = value.value_type.clone();
                let value = self.lower_evaluated_operand(*value, Some(&value_type))?;

                Ok(Rvalue::Array(vec![value; length]))
            }
            ExpressionKind::StructInstantiate(initialise) => {
                let mut fields = Vec::with_capacity(initialise.fields.len());

                for (name, value) in initialise.fields {
                    let value_type = value.value_type.clone();
                    fields.push((
                        name,
                        self.lower_evaluated_operand(value, Some(&value_type))?,
                    ));
                }

                Ok(Rvalue::StructInstantiate {
                    name: initialise.name,
                    fields,
                })
            }
            shape => Err(unsupported_shape(shape)),
        }
    }
}
