use super::*;

impl Walker<'_> {
    pub(super) fn check_expression(&mut self, expression: &Expr) {
        if chain(expression) == 8 {
            self.flag(
                "expression-chain-depth",
                expression.span(),
                "Expression contains eight chained method/nested call stages.",
            );
        }

        if let Expr::Unary(u) = expression
            && matches!(u.op, syn::UnOp::Not(_))
        {
            if matches!(&*u.expr, Expr::Unary(count) if matches!(count.op,syn::UnOp::Not(_))) {
                self.flag(
                    "clarity-extra-double-negation",
                    expression.span(),
                    "Two consecutive unary not operators obscure intent.",
                );
            }

            let inner = if let Expr::Paren(parenthesised_expression) = &*u.expr {
                &*parenthesised_expression.expr
            } else {
                &*u.expr
            };

            if matches!(inner, Expr::Binary(binary_expression) if matches!(binary_expression.op,syn::BinOp::Eq(_) | syn::BinOp::Ne(_)))
            {
                self.flag(
                    "clarity-extra-negated-equality",
                    expression.span(),
                    "Equality/inequality expression is additionally negated.",
                );
            }
        }

        if let Expr::Binary(binary_expression) = expression
            && matches!(binary_expression.op, syn::BinOp::Eq(_) | syn::BinOp::Ne(_))
            && [&*binary_expression.left, &*binary_expression.right]
                .iter()
                .any(|value| matches!(value,Expr::Lit(literal) if matches!(literal.lit,syn::Lit::Bool(_))))
        {
            self.flag(
                "clarity-extra-boolean-comparison",
                expression.span(),
                "Expression compares directly against a Boolean literal.",
            );
        }

        if let Expr::Paren(parenthesised_expression) = expression
            && matches!(&*parenthesised_expression.expr, Expr::Paren(_))
        {
            self.flag(
                "clarity-extra-redundant-parentheses",
                expression.span(),
                "Parentheses directly enclose another parenthesised expression.",
            );
        }

        if let Expr::Tuple(type_value) = expression
            && type_value.elems.len() >= 3
            && type_value.elems
                .iter()
                .all(|element| matches!(element, Expr::Lit(literal) if matches!(literal.lit,syn::Lit::Bool(_))))
        {
            self.flag(
                "clarity-extra-bool-tuple",
                expression.span(),
                "Tuple contains three or more unnamed Boolean literals.",
            );
        }

        if let Expr::Struct(text) = expression
            && text.fields.len() >= 12
        {
            self.flag(
                "clarity-extra-dense-struct-literal",
                expression.span(),
                "Struct literal initialises at least twelve fields.",
            );
        }

        if let Expr::Block(binary_expression) = expression
            && binary_expression.label.is_none()
            && binary_expression.block.stmts.len() == 1
            && matches!(
                &binary_expression.block.stmts[0],
                Stmt::Expr(Expr::Block(_), None)
            )
        {
            self.flag(
                "clarity-extra-redundant-block",
                expression.span(),
                "Unlabelled block only contains another block expression.",
            );
        }

        visit::visit_expr(self, expression);
    }

    pub(super) fn check_pattern_tuple(&mut self, pointer: &syn::PatTuple) {
        if pointer.elems.len() >= 8 {
            self.flag(
                "clarity-extra-large-tuple-pattern",
                pointer.span(),
                "Tuple pattern binds at least eight positions.",
            );
        }

        visit::visit_pat_tuple(self, pointer);
    }

    pub(super) fn check_type_reference(&mut self, type_value: &syn::TypeReference) {
        if matches!(&*type_value.elem,syn::Type::Reference(result) if matches!(&*result.elem,syn::Type::Reference(_)))
        {
            self.flag(
                "clarity-extra-deep-reference-type",
                type_value.span(),
                "Explicit type contains three nested references.",
            );
        }

        visit::visit_type_reference(self, type_value);
    }

    pub(super) fn check_block(&mut self, block: &syn::Block) {
        for pair in block.stmts.windows(2) {
            if let (Stmt::Expr(Expr::If(first_value), _), Stmt::Expr(Expr::If(second_value), _)) =
                (&pair[0], &pair[1])
                && first_value.else_branch.is_none()
                && second_value.else_branch.is_none()
                && end(&first_value.then_branch)
                && tokens(&first_value.then_branch) == tokens(&second_value.then_branch)
            {
                self.flag(
                    "guard-condition-fragmentation",
                    second_value.span(),
                    "Adjacent guards have the same terminal body.",
                );
            }
        }

        visit::visit_block(self, block);
    }
}
