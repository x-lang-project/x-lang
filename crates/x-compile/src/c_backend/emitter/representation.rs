use super::*;

pub(super) fn width(value_type: &Type) -> Option<u32> {
    Some(match value_type {
        Type::I8 | Type::U8 | Type::Character => 8,
        Type::I16 | Type::U16 => 16,
        Type::I32 | Type::U32 => 32,
        Type::I64 | Type::U64 | Type::ISize | Type::USize => 64,
        Type::Boolean => 1,
        _ => return None,
    })
}

pub(super) fn signed(value_type: &Type) -> bool {
    matches!(
        value_type,
        Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::ISize
    )
}
