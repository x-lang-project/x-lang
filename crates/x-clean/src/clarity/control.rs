mod branches;
mod expressions;
mod loops;
use syn::{
    Expr, Stmt,
    spanned::Spanned,
    visit::{self, Visit},
};

use super::tokens;

pub(super) struct Walker<'a> {
    pub source: &'a crate::Source,
    pub findings: &'a mut Vec<crate::Finding>,
    pub depth: usize,
}
fn end(block: &syn::Block) -> bool {
    block.stmts.last().is_some_and(|text| {
        matches!(
            text,
            Stmt::Expr(Expr::Return(_) | Expr::Break(_) | Expr::Continue(_), _)
        )
    })
}

fn body(expression: &Expr) -> Option<&syn::Block> {
    if let Expr::Block(second_value) = expression {
        Some(&second_value.block)
    } else {
        None
    }
}

fn bools(expression: &Expr) -> usize {
    match expression {
        Expr::Binary(binary_expression)
            if matches!(binary_expression.op, syn::BinOp::And(_) | syn::BinOp::Or(_)) =>
        {
            bools(&binary_expression.left) + bools(&binary_expression.right)
        }
        Expr::Paren(parenthesised_expression) => bools(&parenthesised_expression.expr),
        _ => 1,
    }
}

fn chain(expression: &Expr) -> usize {
    match expression {
        Expr::MethodCall(member) => 1 + chain(&member.receiver),
        Expr::Call(call_expression) => {
            1 + call_expression.args.iter().map(chain).max().unwrap_or(0)
        }
        _ => 0,
    }
}

impl Walker<'_> {
    fn flag(&mut self, identifier: &str, span: proc_macro2::Span, message: &str) {
        crate::ReviewSink {
            source: self.source,
            findings: self.findings,
        }
        .at_span(identifier, span, message);
    }

    fn enter(&mut self, span: proc_macro2::Span) {
        self.depth += 1;

        if self.depth == 5 {
            self.flag(
                "deep-control-nesting",
                span,
                "Control flow reaches five nested decisions/loops/closures.",
            );
        }
    }

    fn branches(&mut self, left: &syn::Block, right: &syn::Block, span: proc_macro2::Span) {
        if !left.stmts.is_empty() && tokens(left) == tokens(right) {
            self.flag(
                "identical-sibling-branch-body",
                span,
                "Sibling branches contain identical syntax.",
            );
        }

        if left.stmts.first().is_some_and(|left_value| {
            right
                .stmts
                .first()
                .is_some_and(|result| tokens(left_value) == tokens(result))
        }) {
            self.flag(
                "common-branch-prefix",
                span,
                "Sibling branches start with the same statement.",
            );
        }

        if left.stmts.last().is_some_and(|left_value| {
            right
                .stmts
                .last()
                .is_some_and(|result| tokens(left_value) == tokens(result))
        }) {
            self.flag(
                "common-branch-suffix",
                span,
                "Sibling branches finish with the same statement.",
            );
        }
    }
}

impl<'ast> Visit<'ast> for Walker<'_> {
    fn visit_expr_if(&mut self, expression_value: &'ast syn::ExprIf) {
        self.check_expression_if(expression_value);
    }

    fn visit_expr_match(&mut self, expression_value: &'ast syn::ExprMatch) {
        self.check_expression_match(expression_value);
    }

    fn visit_expr_loop(&mut self, expression_value: &'ast syn::ExprLoop) {
        self.check_expression_loop(expression_value);
    }

    fn visit_expr_for_loop(&mut self, expression_value: &'ast syn::ExprForLoop) {
        self.check_expression_for_loop(expression_value);
    }

    fn visit_expr_while(&mut self, expression_value: &'ast syn::ExprWhile) {
        self.check_expression_while(expression_value);
    }

    fn visit_expr_closure(&mut self, expression_value: &'ast syn::ExprClosure) {
        self.check_expression_closure(expression_value);
    }

    fn visit_expr(&mut self, expression: &'ast Expr) {
        self.check_expression(expression);
    }

    fn visit_pat_tuple(&mut self, pointer: &'ast syn::PatTuple) {
        self.check_pattern_tuple(pointer);
    }

    fn visit_type_reference(&mut self, type_value: &'ast syn::TypeReference) {
        self.check_type_reference(type_value);
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        self.check_block(block);
    }
}

impl Walker<'_> {
    fn loop_body(&mut self, second_value: &syn::Block, label: Option<&syn::Label>) {
        if second_value.stmts.is_empty() {
            self.flag(
                "empty-control-arm",
                second_value.span(),
                "Loop body has no statements.",
            );
        }

        if second_value
            .stmts
            .last()
            .is_some_and(|text| matches!(text, Stmt::Expr(Expr::Return(_) | Expr::Break(_), _)))
        {
            self.flag(
                "single-iteration-loop-shape",
                second_value.span(),
                "Loop body ends in an unconditional break/return; review backedge intent.",
            );
        }

        struct Exits(usize);
        impl<'a> Visit<'a> for Exits {
            fn visit_expr_return(&mut self, _: &'a syn::ExprReturn) {
                self.0 += 1;
            }

            fn visit_expr_break(&mut self, _: &'a syn::ExprBreak) {
                self.0 += 1;
            }

            fn visit_expr_continue(&mut self, _: &'a syn::ExprContinue) {
                self.0 += 1;
            }

            fn visit_expr_closure(&mut self, _: &'a syn::ExprClosure) {}

            fn visit_expr_loop(&mut self, _: &'a syn::ExprLoop) {}

            fn visit_expr_for_loop(&mut self, _: &'a syn::ExprForLoop) {}

            fn visit_expr_while(&mut self, _: &'a syn::ExprWhile) {}
        }
        let mut exits = Exits(0);
        exits.visit_block(second_value);
        if exits.0 >= 4 {
            self.flag(
                "loop-control-exit-density",
                second_value.span(),
                "Loop has four or more exit/continue sites, excluding nested loops and closures.",
            );
        }

        if let Some(left_value) = label {
            let needle = tokens(&left_value.name);

            if !tokens(second_value).contains(&needle) {
                self.flag(
                    "clarity-extra-needless-loop-label",
                    left_value.span(),
                    "Loop label has no explicit syntactic use in its body.",
                );
            }
        }
    }
}
