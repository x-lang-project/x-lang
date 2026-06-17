use super::*;

impl Walker<'_> {
    pub(super) fn check_expression_loop(&mut self, expression_value: &syn::ExprLoop) {
        self.enter(expression_value.span());
        self.loop_body(&expression_value.body, expression_value.label.as_ref());
        visit::visit_expr_loop(self, expression_value);
        self.depth -= 1;
    }

    pub(super) fn check_expression_for_loop(&mut self, expression_value: &syn::ExprForLoop) {
        self.enter(expression_value.span());
        self.loop_body(&expression_value.body, expression_value.label.as_ref());
        visit::visit_expr_for_loop(self, expression_value);
        self.depth -= 1;
    }

    pub(super) fn check_expression_while(&mut self, expression_value: &syn::ExprWhile) {
        self.enter(expression_value.span());
        self.loop_body(&expression_value.body, expression_value.label.as_ref());
        visit::visit_expr_while(self, expression_value);
        self.depth -= 1;
    }

    pub(super) fn check_expression_closure(&mut self, expression_value: &syn::ExprClosure) {
        self.enter(expression_value.span());
        if body(&expression_value.body).is_some_and(|second_value| second_value.stmts.len() >= 8) {
            self.flag(
                "closure-complexity",
                expression_value.span(),
                "Inline closure contains at least eight statements.",
            );
        }

        visit::visit_expr_closure(self, expression_value);
        self.depth -= 1;
    }
}
