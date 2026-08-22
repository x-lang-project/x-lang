use crate::{TypeChecker, errors::TypeError};
use x_ast::{Expression, ExpressionKind, Statement, Type, TypeVisitorMut, UnaryOperator};

struct CheckedTypeNormalizer<'a> {
    checker: &'a mut TypeChecker,
    error: Option<TypeError>,
}

impl CheckedTypeNormalizer<'_> {
    fn validate_integer_literal(&mut self, value: i128, value_type: &Type) {
        let range = match value_type {
            Type::I8 => Some((i128::from(i8::MIN), i128::from(i8::MAX))),
            Type::I16 => Some((i128::from(i16::MIN), i128::from(i16::MAX))),
            Type::I32 => Some((i128::from(i32::MIN), i128::from(i32::MAX))),
            Type::I64 => Some((i128::from(i64::MIN), i128::from(i64::MAX))),
            Type::U8 | Type::Character => Some((0, i128::from(u8::MAX))),
            Type::U16 => Some((0, i128::from(u16::MAX))),
            Type::U32 => Some((0, i128::from(u32::MAX))),
            Type::U64 => Some((0, i128::from(u64::MAX))),
            // Pointer-sized bounds are checked with the target layout.
            _ => None,
        };

        if let Some((minimum, maximum)) = range
            && (value < minimum || value > maximum)
            && self.error.is_none()
        {
            self.error = Some(crate::type_mismatch!(
                format!("integer literal in {minimum}..={maximum} for {value_type}"),
                value.to_string()
            ));
        }
    }
}
