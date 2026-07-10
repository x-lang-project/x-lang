use std::sync::LazyLock;

use pest::{
    iterators::{Pair, Pairs},
    pratt_parser::{Assoc, Op, PrattParser},
};

use super::parse_primary;
use crate::{
    Rule,
    lowering_error::{LoweringError, LoweringResult},
};
use x_ast::{Expression, ExpressionKind, Operator, SourceSpan, Type};
use x_diagnostics::parser_messages::ParserIssue;

static PRATT: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    PrattParser::new()
        .op(Op::infix(Rule::or_operator, Assoc::Left))
        .op(Op::infix(Rule::and_operator, Assoc::Left))
        .op(Op::infix(Rule::bit_or_operator, Assoc::Left))
        .op(Op::infix(Rule::xor_operator, Assoc::Left))
        .op(Op::infix(Rule::bit_and_operator, Assoc::Left))
        .op(Op::infix(Rule::eq_operator, Assoc::Left) | Op::infix(Rule::neq_operator, Assoc::Left))
        .op(Op::infix(Rule::lt_operator, Assoc::Left)
            | Op::infix(Rule::gt_operator, Assoc::Left)
            | Op::infix(Rule::le_operator, Assoc::Left)
            | Op::infix(Rule::ge_operator, Assoc::Left))
        .op(Op::infix(Rule::shl_operator, Assoc::Left) | Op::infix(Rule::shr_operator, Assoc::Left))
        .op(Op::infix(Rule::add_operator, Assoc::Left) | Op::infix(Rule::sub_operator, Assoc::Left))
        .op(Op::infix(Rule::mul_operator, Assoc::Left)
            | Op::infix(Rule::div_operator, Assoc::Left)
            | Op::infix(Rule::mod_operator, Assoc::Left))
        .op(Op::infix(Rule::assign_operator, Assoc::Right))
});

pub(super) fn parse_binary(pairs: Pairs<Rule>) -> LoweringResult<Expression> {
    PRATT
        .map_primary(parse_primary)
        .map_infix(|left, operator, right| lower_infix(left?, operator, right?))
        .parse(pairs)
}

fn lower_infix(
    current_left: Expression,
    current_operator: Pair<Rule>,
    current_right: Expression,
) -> LoweringResult<Expression> {
    let span = SourceSpan::join(current_left.span, current_right.span);

    if current_operator.as_rule() == Rule::assign_operator {
        return Ok(Expression {
            span,
            value_type: Type::Void,
            kind: ExpressionKind::Assignment {
                target: Box::new(current_left),
                value: Box::new(current_right),
            },
        });
    }

    let operator = binary_operator(current_operator.as_rule()).ok_or_else(|| {
        LoweringError::invalid(&current_operator, ParserIssue::UnknownBinaryOperatorRule)
    })?;

    Ok(Expression {
        span,
        value_type: Type::Unknown,
        kind: ExpressionKind::BinaryOperator {
            left: Box::new(current_left),
            operator,
            right: Box::new(current_right),
        },
    })
}

fn binary_operator(rule: Rule) -> Option<Operator> {
    Some(match rule {
        Rule::add_operator => Operator::Add,
        Rule::sub_operator => Operator::Subtract,
        Rule::mul_operator => Operator::Multiply,
        Rule::div_operator => Operator::Divide,
        Rule::mod_operator => Operator::Modulo,
        Rule::shl_operator => Operator::ShiftLeft,
        Rule::shr_operator => Operator::ShiftRight,
        Rule::bit_and_operator => Operator::BitAnd,
        Rule::xor_operator => Operator::Xor,
        Rule::bit_or_operator => Operator::BitOr,
        Rule::lt_operator => Operator::LessThan,
        Rule::gt_operator => Operator::GreaterThan,
        Rule::le_operator => Operator::LessThanOrEqual,
        Rule::ge_operator => Operator::GreaterThanOrEqual,
        Rule::eq_operator => Operator::Equal,
        Rule::neq_operator => Operator::NotEqual,
        Rule::or_operator => Operator::Or,
        Rule::and_operator => Operator::And,
        _ => return None,
    })
}
