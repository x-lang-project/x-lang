use super::*;

pub(in crate::function) fn binary_operand_kinds_are_compatible(
    left: BasicValueEnum<'_>,
    right: BasicValueEnum<'_>,
) -> bool {
    left.get_type() == right.get_type()
        || (left.is_int_value() && right.is_int_value())
        || (left.is_pointer_value() && right.is_int_value())
        || (left.is_int_value() && right.is_pointer_value())
        || (left.is_pointer_value() && right.is_pointer_value())
}

pub(in crate::function) fn is_signed_integer_type(value_type: &Type) -> bool {
    matches!(
        value_type,
        Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::ISize
    )
}

pub(in crate::function) fn signed_integer_minimum_bits(width: u32) -> u64 {
    1_u64 << (width - 1)
}

pub(in crate::function) fn integer_comparison_predicate(
    operator: &Operator,
    signed: bool,
) -> Option<IntPredicate> {
    Some(match (operator, signed) {
        (Operator::LessThan, true) => IntPredicate::SLT,
        (Operator::LessThan, false) => IntPredicate::ULT,
        (Operator::GreaterThan, true) => IntPredicate::SGT,
        (Operator::GreaterThan, false) => IntPredicate::UGT,
        (Operator::LessThanOrEqual, true) => IntPredicate::SLE,
        (Operator::LessThanOrEqual, false) => IntPredicate::ULE,
        (Operator::GreaterThanOrEqual, true) => IntPredicate::SGE,
        (Operator::GreaterThanOrEqual, false) => IntPredicate::UGE,
        _ => return None,
    })
}
