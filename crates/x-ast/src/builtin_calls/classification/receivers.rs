use crate::Type;

pub(super) fn string_receiver(mut value_type: &Type) -> bool {
    while let Type::Reference { inner, .. } = value_type {
        value_type = inner;
    }

    *value_type == Type::String
}

pub(super) fn array_receiver(value_type: &Type) -> bool {
    match value_type {
        Type::Reference { inner, .. } => array_receiver(inner),
        Type::Array(_) | Type::FixedSizedArray(_, _) => true,
        _ => false,
    }
}
