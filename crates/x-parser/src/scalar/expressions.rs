use super::Attempt;
use super::literals::{decode_string, numeric};
use super::name_allowed;
use x_ast::{
    Expression, ExpressionKind, Operator, SourceSpan, StringLiteral, StringPart, Type,
    UnaryOperator,
};

fn binary(text: &str) -> Attempt<(u8, Operator)> {
    Some(match text {
        "||" => (1, Operator::Or),
        "&&" => (2, Operator::And),
        "|" => (3, Operator::BitOr),
        "^" => (4, Operator::Xor),
        "&" => (5, Operator::BitAnd),
        "==" => (6, Operator::Equal),
        "!=" => (6, Operator::NotEqual),
        "<" => (7, Operator::LessThan),
        ">" => (7, Operator::GreaterThan),
        "<=" => (7, Operator::LessThanOrEqual),
        ">=" => (7, Operator::GreaterThanOrEqual),
        "<<" => (8, Operator::ShiftLeft),
        ">>" => (8, Operator::ShiftRight),
        "+" => (9, Operator::Add),
        "-" => (9, Operator::Subtract),
        "*" => (10, Operator::Multiply),
        "/" => (10, Operator::Divide),
        "%" => (10, Operator::Modulo),
        _ => return None,
    })
}
