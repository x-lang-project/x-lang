use pest::iterators::Pair;
use x_ast::Statement;

use crate::{Rule, expression::parse_expr};

pub fn parse_return_statement(pair: Pair<Rule>) -> Statement {
    let mut inner = pair.into_inner();
    let value = inner.next().map(|expr_pair| parse_expr(expr_pair));

    Statement::Return { value }
}
