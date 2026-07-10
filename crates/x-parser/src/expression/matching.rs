use pest::iterators::Pair;

use super::parse_expression;
use crate::{
    Rule, block,
    lowering_error::{LoweringError, LoweringResult},
    source_span_from_pair,
};
use x_ast::{Expression, ExpressionKind, MatchArm, Pattern, Statement, Type};
use x_diagnostics::parser_messages::{ExpectedSyntax, ParserIssue};

pub(super) fn parse_match_expression(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let pair_span = source_span_from_pair(&pair);
    let error_pair = pair.clone();
    let mut inner = pair.into_inner();
    let scrutinee_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::MatchScrutinee))?;
    let scrutinee = parse_expression(scrutinee_pair)?;

    let arms = inner
        .filter(|current_pair| current_pair.as_rule() == Rule::match_arm)
        .map(parse_match_arm)
        .collect::<LoweringResult<_>>()?;

    Ok(Expression {
        span: Some(pair_span),
        value_type: Type::Unknown,
        kind: ExpressionKind::Match {
            scrutinee: Box::new(scrutinee),
            arms,
        },
    })
}

fn parse_match_arm(pair: Pair<Rule>) -> LoweringResult<MatchArm> {
    let error_pair = pair.clone();
    let mut inner = pair.into_inner();
    let pattern_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::MatchPattern))?;
    let pattern = parse_pattern(pattern_pair)?;

    let body_pair = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::MatchArmBody))?;

    match body_pair.as_rule() {
        Rule::block => {
            let body = block::parse_value_block(body_pair)?;

            Ok(MatchArm {
                pattern,
                body,
                result: None,
            })
        }
        Rule::match_return => {
            let expression_pair = body_pair.into_inner().next().ok_or_else(|| {
                LoweringError::missing(&error_pair, ExpectedSyntax::ReturnedExpression)
            })?;

            Ok(MatchArm {
                pattern,
                body: vec![Statement::Return {
                    value: Some(parse_expression(expression_pair)?),
                }],
                result: None,
            })
        }
        Rule::expression => Ok(MatchArm {
            pattern,
            body: Vec::new(),
            result: Some(parse_expression(body_pair)?),
        }),
        _ => Err(LoweringError::invalid(
            &body_pair,
            ParserIssue::UnsupportedMatchArmBody,
        )),
    }
}

fn parse_pattern(pair: Pair<Rule>) -> LoweringResult<Pattern> {
    let error_pair = pair.clone();

    match pair.as_rule() {
        Rule::pattern => parse_wrapped_pattern(pair),
        Rule::enum_pattern => parse_enum_pattern(pair),
        Rule::pattern_path | Rule::identifier => Ok(Pattern::Identifier(pair.as_str().to_string())),
        Rule::tuple_pattern => parse_tuple_pattern(pair),
        _ => Err(LoweringError::invalid(
            &error_pair,
            ParserIssue::UnsupportedPatternRule,
        )),
    }
}

fn parse_wrapped_pattern(pair: Pair<Rule>) -> LoweringResult<Pattern> {
    let error_pair = pair.clone();

    if pair.as_str().trim() == "_" {
        return Ok(Pattern::Wildcard);
    }

    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::InnerPattern))?;

    parse_pattern(inner)
}

fn parse_enum_pattern(pair: Pair<Rule>) -> LoweringResult<Pattern> {
    let error_pair = pair.clone();
    let mut inner = pair.into_inner();
    let variant = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::EnumVariant))?
        .as_str()
        .to_string();

    let mut bindings = Vec::new();

    for binding in inner {
        collect_pattern_bindings(binding, &mut bindings);
    }

    Ok(Pattern::Enum { variant, bindings })
}

fn parse_tuple_pattern(pair: Pair<Rule>) -> LoweringResult<Pattern> {
    let mut bindings = Vec::new();

    for binding in pair.into_inner() {
        if binding
            .clone()
            .into_inner()
            .any(|child| child.as_rule() == Rule::tuple_pattern)
        {
            return Err(LoweringError::invalid(
                &binding,
                ParserIssue::NestedTuplePattern,
            ));
        }

        bindings.push(binding.as_str().to_string());
    }

    Ok(Pattern::Tuple(bindings))
}

fn collect_pattern_bindings(pair: Pair<Rule>, bindings: &mut Vec<String>) {
    match pair.as_rule() {
        Rule::pattern_argument if pair.as_str() == "_" => bindings.push("_".to_string()),
        Rule::pattern_argument | Rule::tuple_pattern => {
            for child in pair.into_inner() {
                collect_pattern_bindings(child, bindings);
            }
        }
        Rule::pattern_path | Rule::identifier => {
            bindings.push(pair.as_str().to_string());
        }
        _ => {}
    }
}
