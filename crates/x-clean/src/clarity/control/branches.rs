use super::*;

impl Walker<'_> {
    pub(super) fn check_expression_if(&mut self, expression_value: &syn::ExprIf) {
        self.enter(expression_value.span());
        if bools(&expression_value.cond) >= 5 {
            self.flag(
                "boolean-condition-density",
                expression_value.cond.span(),
                "Condition has at least five Boolean operands.",
            );
        }

        if matches!(&*expression_value.cond, Expr::Lit(literal) if matches!(literal.lit, syn::Lit::Bool(_)))
        {
            self.flag(
                "clarity-extra-literal-condition",
                expression_value.cond.span(),
                "If condition is a Boolean literal.",
            );
        }

        if expression_value.then_branch.stmts.is_empty() {
            self.flag(
                "empty-control-arm",
                expression_value.then_branch.span(),
                "If branch has no statements.",
            );
        }

        if let Some((_, otherwise)) = &expression_value.else_branch {
            if end(&expression_value.then_branch) {
                self.flag("else-after-terminal-branch", expression_value.span(), "Then branch ends in an unconditional exit, but an else still wraps the alternative.");
            }

            if let Some(second_value) = body(otherwise) {
                self.branches(
                    &expression_value.then_branch,
                    second_value,
                    expression_value.span(),
                );
                if second_value.stmts.is_empty() {
                    self.flag(
                        "clarity-extra-empty-else",
                        second_value.span(),
                        "Explicit else block contains no statements.",
                    );
                }

                if second_value.stmts.len() == 1
                    && matches!(&second_value.stmts[0], Stmt::Expr(Expr::If(_), _))
                {
                    self.flag(
                        "clarity-extra-nested-if-else",
                        second_value.span(),
                        "Else block only encloses another if expression.",
                    );
                }

                let literal = tokens(&expression_value.then_branch);
                let result = tokens(second_value);

                if (literal == "{ true }" && result == "{ false }")
                    || (literal == "{ return true ; }" && result == "{ return false ; }")
                {
                    self.flag(
                        "clarity-extra-boolean-return-if",
                        expression_value.span(),
                        "Branches return true and false directly from the condition.",
                    );
                }
            }

            let mut seen = std::collections::HashSet::from([tokens(&expression_value.cond)]);
            let mut next = otherwise.as_ref();

            while let Expr::If(count) = next {
                if !seen.insert(tokens(&count.cond)) {
                    self.flag(
                        "repeated-condition-in-branch-chain",
                        count.cond.span(),
                        "Condition repeats earlier syntax in this if/else-if chain.",
                    );
                }

                if let Some((_, count)) = &count.else_branch {
                    next = count;
                } else {
                    break;
                }
            }
        }

        visit::visit_expr_if(self, expression_value);
        self.depth -= 1;
    }

    pub(super) fn check_expression_match(&mut self, expression_value: &syn::ExprMatch) {
        self.enter(expression_value.span());
        if expression_value.arms.len() >= 20 {
            self.flag(
                "match-arm-count-outlier",
                expression_value.span(),
                "Match contains at least twenty arms.",
            );
        }

        let mut seen = std::collections::HashSet::new();
        let mut patterns = std::collections::HashSet::new();

        for arm in &expression_value.arms {
            if !patterns.insert(tokens(&arm.pat)) {
                self.flag(
                    "clarity-extra-duplicate-match-pattern",
                    arm.pat.span(),
                    "An earlier match arm has the same pattern syntax.",
                );
            }

            if !seen.insert(tokens(&arm.body)) {
                self.flag(
                    "identical-sibling-branch-body",
                    arm.body.span(),
                    "Earlier match arm contains the same body syntax.",
                );
            }

            if body(&arm.body).is_some_and(|second_value| second_value.stmts.is_empty()) {
                self.flag(
                    "empty-control-arm",
                    arm.span(),
                    "Match arm has an empty body.",
                );
            }

            if arm
                .guard
                .as_ref()
                .is_some_and(|(_, expression_value)| bools(expression_value) >= 4)
            {
                self.flag(
                    "clarity-extra-complex-match-guard",
                    arm.span(),
                    "Match guard combines at least four Boolean operands.",
                );
            }
        }

        if expression_value.arms.len() == 2
            && expression_value.arms
                .iter()
                .any(|first_value| matches!(&*first_value.body, Expr::Match(member) if member.arms.len() == 2))
            && (tokens(expression_value).contains("Some") || tokens(expression_value).contains("Ok"))
        {
            self.flag(
                "nested-single-option-match",
                expression_value.span(),
                "Two-arm Some/None or Ok/Err-shaped match nests another two-arm match.",
            );
        }

        visit::visit_expr_match(self, expression_value);
        self.depth -= 1;
    }
}
