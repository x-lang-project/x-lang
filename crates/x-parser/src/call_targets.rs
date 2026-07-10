use pest::iterators::Pair;

use crate::{
    Rule,
    lowering_error::{LoweringError, LoweringResult},
    types::parse_type,
};
use x_ast::{CallPath, CallSegment, CallTarget, Type};
use x_diagnostics::parser_messages::ExpectedSyntax;

pub(super) fn parse_target(pair: Pair<Rule>) -> LoweringResult<CallTarget> {
    if pair.as_rule() != Rule::path {
        return Ok(CallTarget::plain(pair.as_str().trim()));
    }

    let segments = pair
        .into_inner()
        .map(parse_segment)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CallTarget::new(CallPath { segments }))
}

fn parse_segment(pair: Pair<Rule>) -> LoweringResult<CallSegment> {
    let error_pair = pair.clone();
    let mut children = pair.into_inner();
    let name = children
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::CallableName))?
        .as_str()
        .to_owned();
    let type_arguments = children
        .find(|pair| pair.as_rule() == Rule::generic_arguments)
        .map(parse_arguments)
        .transpose()?
        .unwrap_or_default();

    Ok(CallSegment {
        name,
        type_arguments,
    })
}

pub(super) fn parse_arguments(pair: Pair<Rule>) -> LoweringResult<Vec<Type>> {
    pair.into_inner().map(parse_type).collect()
}

pub(super) fn attach_leaf_arguments(
    target: &mut CallTarget,
    pair: Pair<Rule>,
) -> LoweringResult<()> {
    let arguments = parse_arguments(pair.clone())?;
    let segment = target
        .path
        .segments
        .last_mut()
        .ok_or_else(|| LoweringError::missing(&pair, ExpectedSyntax::CallableName))?;
    segment.type_arguments = arguments;

    Ok(())
}
