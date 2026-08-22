use super::*;
use x_diagnostics::frontend_messages::FrontendMessage;

impl TypeChecker {
    pub(super) fn check_array(&mut self, elements: &mut [Expression]) -> Result<Type, TypeError> {
        let Some((first, remaining)) = elements.split_first_mut() else {
            return Err(TypeError::lookup(FrontendMessage::UnboundEmptyArray));
        };

        let first_type = self.check_expression(first)?;

        for element in remaining {
            let next_type = self.check_expression(element)?;

            if !self.types_compatible(&first_type, &next_type) {
                return Err(crate::type_mismatch!(
                    format!("{:?}", first_type),
                    format!("{:?}", next_type)
                ));
            }
        }

        Ok(Type::FixedSizedArray(Box::new(first_type), elements.len()))
    }

    pub(super) fn check_array_repeat(
        &mut self,
        value: &mut Expression,
        expression: &mut Expression,
    ) -> Result<Type, TypeError> {
        let element_type = self.check_expression(value)?;
        let count_type = self.check_expression(expression)?;

        if !self.is_integer_primitive(&count_type) {
            return Err(crate::type_mismatch!(
                "a non-negative compile-time integer array length",
                format!("{count_type:?}")
            ));
        }

        let ExpressionKind::Integer(raw_count) = expression.kind else {
            return Err(TypeError::inference(
                FrontendMessage::NonConstantArrayRepetition,
            ));
        };

        let length = usize::try_from(raw_count)
            .map_err(|_| TypeError::inference(FrontendMessage::InvalidArrayRepetitionLength))?;

        value.value_type = element_type.clone();
        expression.value_type = Type::USize;

        Ok(Type::FixedSizedArray(Box::new(element_type), length))
    }

    pub(super) fn check_tuple(&mut self, elements: &mut [Expression]) -> Result<Type, TypeError> {
        if elements.is_empty() {
            Ok(Type::Void)
        } else {
            let element_types = elements
                .iter_mut()
                .map(|element| self.check_expression(element))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Type::Tuple(element_types))
        }
    }

    pub(super) fn check_array_access(
        &mut self,
        array: &mut Expression,
        index: &mut Expression,
    ) -> Result<Type, TypeError> {
        let array_type = self.check_expression(array)?;
        let index_type = self.check_expression(index)?;

        array.value_type = array_type.clone();
        index.value_type = index_type.clone();

        let ibase = self.resolve_type(&index_type)?;

        if !matches!(ibase, Type::I32 | Type::I64 | Type::USize | Type::ISize)
            && ibase != Type::Never
        {
            return Err(crate::type_mismatch!(
                "i32, i64, isize or usize index",
                format!("{:?}", ibase)
            ));
        }

        let mut array_base = self.resolve(&array_type);

        while let Type::Reference { inner, .. } = array_base {
            array_base = *inner;
        }

        match array_base {
            Type::Array(inner) => Ok(*inner),
            Type::FixedSizedArray(inner, length) => {
                if let ExpressionKind::Integer(value) = index.kind
                    && (value < 0
                        || usize::try_from(value).map_or(true, |element| element >= length))
                {
                    return Err(TypeError::ConstantIndexOutOfBounds {
                        index: value,
                        length,
                    });
                }

                Ok(*inner)
            }
            Type::RawPointer { inner, .. } => Ok(*inner),
            other => Err(crate::type_mismatch!(
                "array, fixed-size array, or raw pointer",
                format!("{other:?}")
            )),
        }
    }
}
