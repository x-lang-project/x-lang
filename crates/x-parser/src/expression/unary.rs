use pest::iterators::Pair;

use crate::{
    Rule,
    lowering_error::{LoweringError, LoweringResult},
    postfix_operator::parse_postfix_operator,
    source_span_from_pair, term,
};
use x_ast::{Expression, ExpressionKind, SourceSpan, Type, UnaryOperator};
use x_diagnostics::parser_messages::{ExpectedSyntax, ParserIssue};

pub fn parse_unary(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let pair_span = source_span_from_pair(&pair);

    Ok(parse_unary_expression(pair)?.with_span(pair_span))
}

fn parse_unary_expression(pair: Pair<Rule>) -> LoweringResult<Expression> {
    if pair.as_rule() != Rule::unary_expression {
        return Err(LoweringError::invalid(
            &pair,
            ParserIssue::ExpectedUnaryExpression,
        ));
    }

    let pair_span = source_span_from_pair(&pair);
    let error_pair = pair.clone();
    let mut inner = pair.into_inner();
    let first = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::UnaryOperand))?;

    match first.as_rule() {
        Rule::neg_operator
        | Rule::not_operator
        | Rule::bit_not_operator
        | Rule::pre_inc_operator
        | Rule::pre_dec_operator => {
            let operator = prefix_unary_operator(first.as_rule()).ok_or_else(|| {
                LoweringError::invalid(&first, ParserIssue::UnknownPrefixUnaryOperator)
            })?;
            let right_pair = inner.next().ok_or_else(|| {
                LoweringError::missing(&error_pair, ExpectedSyntax::PrefixOperand)
            })?;
            let right = parse_unary_expression(right_pair)?;

            Ok(Expression {
                span: SourceSpan::join(Some(pair_span), right.span),
                value_type: Type::Unknown,
                kind: ExpressionKind::UnaryOperator {
                    operator,
                    expression: Box::new(right),
                },
            })
        }
        Rule::address_of => term::parse_address_of(first, &error_pair),
        Rule::deref => term::parse_dereference(first, &error_pair),
        Rule::postfix_expression => parse_postfix_operator(first),
        _ => Err(LoweringError::invalid(
            &first,
            ParserIssue::UnsupportedUnaryExpressionChild,
        )),
    }
}

fn prefix_unary_operator(rule: Rule) -> Option<UnaryOperator> {
    Some(match rule {
        Rule::neg_operator => UnaryOperator::Negate,
        Rule::not_operator => UnaryOperator::LogicalNot,
        Rule::bit_not_operator => UnaryOperator::BitwiseNot,
        Rule::pre_inc_operator => UnaryOperator::PreIncrement,
        Rule::pre_dec_operator => UnaryOperator::PreDecrement,
        _ => return None,
    })
}
