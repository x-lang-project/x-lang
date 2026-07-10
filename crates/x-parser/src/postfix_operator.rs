use pest::iterators::Pair;

use crate::function;
use crate::{
    Rule,
    expression::parse_expression,
    lowering_error::{LoweringError, LoweringResult},
    source_span_from_pair,
    term::parse_term,
};
use x_ast::SourceSpan;
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::parser_messages::ExpectedSyntax;
use x_diagnostics::parser_messages::ParserIssue;

pub fn parse_postfix_operator(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let error_pair = pair.clone();
    let mut inner = pair.into_inner();

    let base_pair = inner.next().ok_or_else(|| {
        LoweringError::missing(&error_pair, ExpectedSyntax::PostfixBaseExpression)
    })?;
    let mut expression = parse_term(base_pair)?;

    for post in inner {
        expression = parse_postfix(expression, post)?;
    }

    Ok(expression)
}

pub fn parse_postfix(base: Expression, postfix: Pair<Rule>) -> LoweringResult<Expression> {
    let postfix_span = source_span_from_pair(&postfix);
    let error_pair = postfix.clone();
    let inner = postfix
        .into_inner()
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::PostfixOperation))?;
    let span = SourceSpan::join(base.span, Some(postfix_span));
    let kind = match inner.as_rule() {
        Rule::array_access => parse_array_access(base, inner, &error_pair)?,
        Rule::field_access => return parse_field_access(base, inner, &error_pair, span),
        Rule::call_postfix => parse_call_postfix(base, inner)?,
        Rule::method_call => parse_method_call(base, inner, &error_pair)?,
        Rule::try_postfix => ExpressionKind::Try {
            expression: Box::new(base),
        },

        _ => {
            return Err(LoweringError::invalid(
                &inner,
                ParserIssue::UnsupportedPostfixOperation,
            ));
        }
    };

    Ok(Expression {
        span,
        value_type: Type::Unknown,
        kind,
    })
}

fn parse_field_access(
    base: Expression,
    inner: Pair<Rule>,
    error_pair: &Pair<Rule>,
    span: Option<SourceSpan>,
) -> LoweringResult<Expression> {
    let identifier = inner
        .into_inner()
        .next()
        .ok_or_else(|| LoweringError::missing(error_pair, ExpectedSyntax::FieldName))?;
    let field = identifier.as_str().trim().to_string();
    let (value_type, kind) = if field == "char" {
        (
            Type::Character,
            ExpressionKind::Cast {
                expression: Box::new(base),
                to: Type::Character,
            },
        )
    } else {
        (
            Type::Unknown,
            ExpressionKind::FieldAccess {
                object: Box::new(base),
                field,
            },
        )
    };

    Ok(Expression {
        span,
        value_type,
        kind,
    })
}

fn parse_array_access(
    base: Expression,
    inner: Pair<Rule>,
    error_pair: &Pair<Rule>,
) -> LoweringResult<ExpressionKind> {
    let index_pair = inner
        .into_inner()
        .next()
        .ok_or_else(|| LoweringError::missing(error_pair, ExpectedSyntax::ArrayIndex))?;
    let index = parse_expression(index_pair)?;

    Ok(ExpressionKind::ArrayAccess {
        array: Box::new(base),
        index: Box::new(index),
    })
}

fn parse_call_postfix(base: Expression, inner: Pair<Rule>) -> LoweringResult<ExpressionKind> {
    let ExpressionKind::Identifier(callee) = &base.kind else {
        return Err(LoweringError::invalid(
            &inner,
            ParserIssue::UnsupportedPostfixOperation,
        ));
    };

    let callee = callee.clone();
    let arguments = function::parse_arguments(
        inner
            .into_inner()
            .find(|pair| pair.as_rule() == Rule::arguments),
    )?;

    Ok(ExpressionKind::FunctionCall {
        target: callee.into(),
        arguments,
    })
}

fn parse_method_call(
    base: Expression,
    inner: Pair<Rule>,
    error_pair: &Pair<Rule>,
) -> LoweringResult<ExpressionKind> {
    let mut inner_pairs = inner.into_inner().peekable();
    let identifier = inner_pairs
        .next()
        .ok_or_else(|| LoweringError::missing(error_pair, ExpectedSyntax::MethodName))?;
    let mut target = x_ast::CallTarget::plain(identifier.as_str());

    if let Some(next_pair) = inner_pairs.peek()
        && next_pair.as_rule() == Rule::generic_arguments
    {
        crate::call_targets::attach_leaf_arguments(&mut target, next_pair.clone())?;
        inner_pairs.next();
    }

    let arguments_pair = inner_pairs.find(|pair| pair.as_rule() == Rule::arguments);
    let arguments = function::parse_arguments(arguments_pair)?;

    Ok(ExpressionKind::MethodCall {
        object: Box::new(base),
        target,
        arguments,
    })
}
