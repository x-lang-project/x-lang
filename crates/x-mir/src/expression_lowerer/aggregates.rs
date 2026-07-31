use super::ExpressionLowerer;
use crate::errors::MirLowerError;
use crate::lower;
use crate::lower::canonical_array_expression_type;
use crate::{Operand, Rvalue};
use x_ast::{Expression, StructInitialisation, Type};

impl ExpressionLowerer<'_> {
    pub(super) fn lower_struct_value(
        &mut self,
        initialise: StructInitialisation,
        expected_type: Option<&Type>,
    ) -> Result<Operand, MirLowerError> {
        let mut lowered_fields = Vec::with_capacity(initialise.fields.len());

        for (field_name, field_expression) in initialise.fields {
            let expected = field_expression.value_type.clone();
            let operand = self.lower_expression_with_context(field_expression, Some(&expected))?;
            lowered_fields.push((field_name, operand));
        }

        let temporary_type = expected_type
            .cloned()
            .unwrap_or_else(|| Type::Custom(initialise.name.clone()));
        let struct_name = match &temporary_type {
            Type::Custom(name) | Type::GenericInstance { name, .. } => name.clone(),
            _ => initialise.name,
        };

        Ok(self.emit_temporary(
            temporary_type,
            Rvalue::StructInstantiate {
                name: struct_name,
                fields: lowered_fields,
            },
        ))
    }

    pub(super) fn lower_tuple_value(
        &mut self,
        elements: Vec<Expression>,
        result_type: Type,
    ) -> Result<Operand, MirLowerError> {
        let mut lowered = Vec::with_capacity(elements.len());

        for element in elements {
            let expected = element.value_type.clone();
            lowered.push(self.lower_expression_with_context(element, Some(&expected))?);
        }

        Ok(self.emit_temporary(result_type.clone(), Rvalue::Tuple(lowered)))
    }

    pub(super) fn lower_array_value(
        &mut self,
        elements: Vec<Expression>,
        result_type: Type,
        expected_type: Option<&Type>,
    ) -> Result<Operand, MirLowerError> {
        let array_type = expected_type
            .cloned()
            .unwrap_or_else(|| canonical_array_expression_type(&result_type, elements.len()));
        let element_type = match &array_type {
            Type::FixedSizedArray(inner, _) | Type::Array(inner) => Some((**inner).clone()),
            _ => None,
        };

        let mut lowered = Vec::with_capacity(elements.len());

        for element in elements {
            let expected = element_type
                .clone()
                .unwrap_or_else(|| element.value_type.clone());
            lowered.push(self.lower_expression_with_context(element, Some(&expected))?);
        }

        Ok(self.emit_temporary(array_type, Rvalue::Array(lowered)))
    }

    pub(super) fn lower_repeated_array_value(
        &mut self,
        value: Expression,
        expression: Expression,
        expected_type: Option<&Type>,
    ) -> Result<Operand, MirLowerError> {
        let length = lower::array_repeat_length(&expression)?;
        let element_type = value.value_type.clone();
        let array_type = expected_type
            .cloned()
            .unwrap_or_else(|| Type::FixedSizedArray(Box::new(element_type.clone()), length));
        let value = self.lower_expression_with_context(value, Some(&element_type))?;

        Ok(self.emit_temporary(array_type, Rvalue::Array(vec![value; length])))
    }
}
