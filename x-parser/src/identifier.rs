use crate::Rule;
use pest::iterators::Pair;
use x_ast::Expr;

pub fn parse_identifier(pair: Pair<Rule>) -> Expr {
    Expr::Identifier(pair.as_str().to_string())
}
