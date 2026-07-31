use super::ExpressionLowerer;
use crate::Operand;
use x_ast::Type;

pub(super) use crate::lower::operand_type;
pub(super) use crate::source_names::method_call_function_name;

impl ExpressionLowerer<'_> {
    pub(super) fn dereference_type(
        &self,
        operand: &Operand,
        source: &Type,
        expected: Type,
    ) -> Type {
        if expected != Type::Unknown {
            return expected;
        }

        operand_type(operand, self.locals.declarations())
            .as_ref()
            .and_then(Type::pointee)
            .or_else(|| source.pointee())
            .cloned()
            .unwrap_or(Type::Unknown)
    }
}
