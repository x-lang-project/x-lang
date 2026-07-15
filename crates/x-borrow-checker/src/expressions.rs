use super::{BorrowCheckError, BorrowChecker, ReferenceValue};
use x_ast::SourceError;
use x_ast::{Expression, ExpressionKind, Type};

impl BorrowChecker {
    fn resolve_reborrow_owner(
        &self,
        current_expression: &Expression,
    ) -> Option<(String, Option<String>)> {
        self.resolve_lvalue_owner_expression(current_expression)
            .or_else(|| match &current_expression.kind {
                ExpressionKind::Dereference { expression }
                    if self.is_safe_reference_value(expression) =>
                {
                    self.resolve_reborrow_owner(expression)
                }
                _ => None,
            })
    }
}
