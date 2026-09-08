use super::*;

fn literal_array<'ctx>(
    value_type: types::BasicTypeEnum<'ctx>,
    values: &[BasicValueEnum<'ctx>],
) -> Option<BasicValueEnum<'ctx>> {
    use inkwell::types::BasicTypeEnum;

    macro_rules! array {
        ($ty:expr, $predicate:ident, $conversion:ident) => {
            if values.iter().all(|value| value.$predicate()) {
                Some(
                    $ty.const_array(
                        &values
                            .iter()
                            .map(|value| value.$conversion())
                            .collect::<Vec<_>>(),
                    )
                    .into(),
                )
            } else {
                None
            }
        };
    }
    match value_type {
        BasicTypeEnum::IntType(value_type) => array!(value_type, is_int_value, into_int_value),
        BasicTypeEnum::FloatType(value_type) => {
            array!(value_type, is_float_value, into_float_value)
        }
        BasicTypeEnum::PointerType(value_type) => {
            array!(value_type, is_pointer_value, into_pointer_value)
        }
        BasicTypeEnum::StructType(value_type) => {
            array!(value_type, is_struct_value, into_struct_value)
        }
        BasicTypeEnum::ArrayType(value_type) => {
            array!(value_type, is_array_value, into_array_value)
        }
        BasicTypeEnum::VectorType(value_type) => {
            array!(value_type, is_vector_value, into_vector_value)
        }
        BasicTypeEnum::ScalableVectorType(_) => None,
    }
}
