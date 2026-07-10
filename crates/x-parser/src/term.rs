use pest::iterators::Pair;

use crate::block;
use crate::{
    Rule,
    array::parse_array_literal,
    boolean::parse_boolean,
    expression::{parse_expression, parse_unary},
    function::parse_function_call,
    identifier::parse_identifier,
    lowering_error::{LoweringError, LoweringResult},
    number::parse_number,
    postfix_operator::parse_postfix_operator,
    source_span_from_pair,
    string::parse_string,
    r#struct::parse_struct_instantiate,
    types::parse_type,
    with_pair_span,
};
use x_ast::SourceSpan;
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::parser_messages::ExpectedSyntax;
use x_diagnostics::parser_messages::ParserIssue;

pub fn parse_term(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let error_pair = pair.clone();
    let rule = pair.as_rule();
    let base = match rule {
        Rule::unary_expression => parse_unary(pair)?,
        Rule::address_of => parse_address_of(pair, &error_pair)?,
        Rule::deref => parse_dereference(pair, &error_pair)?,
        Rule::postfix_expression => parse_postfix_operator(pair)?,
        Rule::primary => {
            let inner_pair = pair.into_inner().next().ok_or_else(|| {
                LoweringError::missing(&error_pair, ExpectedSyntax::PrimaryExpression)
            })?;

            parse_term(inner_pair)?
        }
        Rule::function_call => parse_function_call(pair)?,
        Rule::path | Rule::identifier | Rule::value_identifier | Rule::type_identifier => {
            parse_identifier(pair)
        }
        Rule::number => parse_number(pair)?,
        Rule::string => parse_string(pair)?,
        Rule::char_literal => parse_char_literal(pair)?,
        Rule::boolean => parse_boolean(pair),
        Rule::expression | Rule::if_expression | Rule::match_expression => parse_expression(pair)?,
        Rule::array_literal => parse_array_literal(pair)?,
        Rule::tuple_literal => parse_tuple_literal(pair)?,
        Rule::struct_instantiate => parse_struct_instantiate(pair)?,
        Rule::unsafe_block => parse_unsafe_block_expression(pair)?,
        Rule::sizeof_expression => parse_sizeof(pair)?,
        Rule::cast_expression => parse_cast(pair)?,
        _ => {
            return Err(LoweringError::invalid(
                &error_pair,
                ParserIssue::UnsupportedTermRule,
            ));
        }
    };

    Ok(with_pair_span(base, &error_pair))
}

fn parse_sizeof(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let error_pair = pair.clone();
    let type_pair = pair
        .into_inner()
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::SizeofType))?;

    Ok(Expression {
        span: Some(source_span_from_pair(&error_pair)),
        value_type: Type::USize,
        kind: ExpressionKind::SizeOf(parse_type(type_pair)?),
    })
}

fn parse_cast(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let pair_span = source_span_from_pair(&pair);
    let error_pair = pair.clone();
    let mut inner = pair.into_inner();
    let base_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::CastOperand))?;
    let mut base = parse_term(base_pair)?;

    for cast_pair in inner {
        let type_pair = cast_pair
            .into_inner()
            .next()
            .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::CastTargetType))?;
        let value_type = parse_type(type_pair)?;
        base = Expression {
            span: SourceSpan::join(base.span, Some(pair_span)),
            value_type: value_type.clone(),
            kind: ExpressionKind::Cast {
                expression: Box::new(base),
                to: value_type,
            },
        };
    }

    Ok(base)
}

pub(crate) fn parse_address_of(
    pair: Pair<Rule>,
    error_pair: &Pair<Rule>,
) -> LoweringResult<Expression> {
    let pair_span = source_span_from_pair(error_pair);
    let mut is_mut = false;
    let mut is_unique = false;
    let mut target_pair = None;

    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::kw_mut => is_mut = true,
            Rule::unique_kw => is_unique = true,
            _ => target_pair = Some(child),
        }
    }

    let term_pair = target_pair
        .ok_or_else(|| LoweringError::missing(error_pair, ExpectedSyntax::AddressTarget))?;
    let target = parse_term(term_pair)?;

    Ok(Expression {
        span: SourceSpan::join(Some(pair_span), target.span),
        value_type: Type::Unknown,
        kind: ExpressionKind::AddressOf {
            is_mut,
            is_unique,
            expression: Box::new(target),
        },
    })
}

pub(crate) fn parse_dereference(
    pair: Pair<Rule>,
    error_pair: &Pair<Rule>,
) -> LoweringResult<Expression> {
    let pair_span = source_span_from_pair(error_pair);
    let mut inner = pair.into_inner();
    let term_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(error_pair, ExpectedSyntax::DereferenceTarget))?;
    let target = parse_term(term_pair)?;

    Ok(Expression {
        span: SourceSpan::join(Some(pair_span), target.span),
        value_type: Type::Unknown,
        kind: ExpressionKind::Dereference {
            expression: Box::new(target),
        },
    })
}

fn parse_unsafe_block_expression(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let pair_span = source_span_from_pair(&pair);
    let (statements, result) = block::parse_expression_block(pair)?;

    Ok(Expression {
        span: Some(pair_span),
        value_type: Type::Unknown,
        kind: ExpressionKind::Block { statements, result },
    })
}

fn parse_tuple_literal(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let pair_span = source_span_from_pair(&pair);
    let elements: Vec<Expression> = pair
        .into_inner()
        .map(|current_pair| parse_expression(current_pair))
        .collect::<LoweringResult<_>>()?;

    Ok(Expression {
        span: Some(pair_span),
        value_type: Type::Unknown,
        kind: ExpressionKind::Tuple(elements),
    })
}

fn parse_char_literal(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let pair_span = source_span_from_pair(&pair);
    let text = pair.as_str();
    let inner = if text.len() >= 2 {
        &text[1..text.len() - 1]
    } else {
        ""
    };

    let ch = if inner.starts_with("\\u{") {
        let hex = &inner[3..inner.len() - 1];

        u32::from_str_radix(hex, 16)
            .ok()
            .and_then(char::from_u32)
            .ok_or_else(|| LoweringError::invalid(&pair, ParserIssue::InvalidUnicodeScalarValue))?
    } else if inner.starts_with('\\') {
        match inner.chars().nth(1) {
            Some('n') => '\n',
            Some('t') => '\t',
            Some('r') => '\r',
            Some('0') => '\0',
            Some('\\') => '\\',
            Some('\'') => '\'',
            Some('"') => '"',
            Some(character) => character,
            None => {
                return Err(LoweringError::invalid(
                    &pair,
                    ParserIssue::IncompleteCharacterEscape,
                ));
            }
        }
    } else {
        inner
            .chars()
            .next()
            .ok_or_else(|| LoweringError::missing(&pair, ExpectedSyntax::CharacterValue))?
    };

    Ok(Expression {
        span: Some(pair_span),
        value_type: Type::Character,
        kind: ExpressionKind::Character(ch),
    })
}
