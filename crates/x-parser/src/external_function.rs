use pest::iterators::Pair;

use crate::{
    Rule,
    lowering_error::{LoweringError, LoweringResult},
};
use x_ast::{ExternParameter, Statement};
use x_diagnostics::parser_messages::ExpectedSyntax;
use x_diagnostics::parser_messages::ParserIssue;

pub fn parse_external_function_declaration(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let error_pair = pair.clone();
    let inner = pair.into_inner();
    let mut parts: Vec<Pair<Rule>> = Vec::new();
    let mut is_pure = false;
    let mut abi = None;

    for current_pair in inner {
        if current_pair.as_rule() == Rule::external_modifier {
            is_pure |= current_pair.as_str().trim() == "pure";
            continue;
        }

        parts.push(current_pair);
    }

    if parts.is_empty() {
        return Err(LoweringError::missing(
            &error_pair,
            ExpectedSyntax::ExternalFunctionDeclaration,
        ));
    }

    let name_index = usize::from(
        parts
            .first()
            .is_some_and(|part| is_abi_rule(part.as_rule()))
            && parts
                .get(1)
                .is_some_and(|part| part.as_rule() == Rule::identifier),
    );

    let name_pair = parts
        .get(name_index)
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::ExternalFunctionName))?;
    let name = name_pair.as_str().to_string();

    if name_index > 0 {
        let abi_pair = parts
            .first()
            .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::AbiName))?;
        abi = Some(abi_pair.as_str().trim().to_string());
    }

    let mut parameters: Vec<ExternParameter> = Vec::new();
    let mut return_type: Option<String> = None;

    let mut signature = parts.into_iter().skip(name_index);
    let _declared_name = signature
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::ExternalFunctionName))?;

    for current_pair in signature {
        match current_pair.as_rule() {
            Rule::external_parameters => {
                parameters.extend(
                    current_pair
                        .into_inner()
                        .map(|parameter| parse_external_parameter(parameter, &error_pair))
                        .collect::<LoweringResult<Vec<_>>>()?,
                );
            }
            Rule::type_specifier => {
                return_type = Some(current_pair.as_str().to_string());
            }
            _ => {}
        }
    }

    Ok(Statement::ExternFunctionDeclaration {
        name,
        parameters,
        return_type,
        abi,
        is_pure,
    })
}

fn parse_external_parameter(
    pair: Pair<Rule>,
    error_pair: &Pair<Rule>,
) -> LoweringResult<ExternParameter> {
    // Pest omits the variadic literal from child pairs.
    let is_variadic = pair.as_str().trim_end().ends_with("...");
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .ok_or_else(|| LoweringError::missing(error_pair, ExpectedSyntax::ExternalParameterName))?
        .as_str()
        .to_string();
    let type_name = match inner.next() {
        Some(value_type) => value_type.as_str().to_string(),
        None if is_variadic => "...".to_string(),
        None => {
            return Err(LoweringError::missing(
                error_pair,
                ExpectedSyntax::ExternalParameterType,
            ));
        }
    };

    Ok(ExternParameter { name, type_name })
}

pub fn parse_external_block(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let mut inner = pair.into_inner();
    let mut abi = None;

    if let Some(first) = inner.peek()
        && is_abi_rule(first.as_rule())
    {
        abi = inner.next().map(|abi| abi.as_str().trim().to_string());
    }

    let mut current_statements = Vec::new();

    for item in inner {
        if item.as_rule() != Rule::external_block_function {
            return Err(LoweringError::invalid(
                &item,
                ParserIssue::ExternalBlockDeclaration,
            ));
        }

        current_statements.push(parse_external_block_function(item, abi.clone())?);
    }

    Ok(match current_statements.pop() {
        Some(statement) if current_statements.is_empty() => statement,
        last => {
            current_statements.extend(last);

            Statement::Block {
                statements: current_statements,
            }
        }
    })
}

fn parse_external_block_function(
    item: Pair<Rule>,
    abi: Option<String>,
) -> LoweringResult<Statement> {
    let item_error = item.clone();
    let mut name = None;
    let mut parameters = Vec::new();
    let mut return_type = None;
    let mut is_pure = false;

    for rem in item.into_inner() {
        match rem.as_rule() {
            Rule::external_modifier => {
                is_pure |= rem.as_str().trim() == "pure";
            }
            Rule::identifier => name = Some(rem.as_str().to_string()),
            Rule::function_parameter_list => {
                parameters = parse_external_block_parameters(rem, &item_error)?;
            }
            Rule::return_type_annotation => {
                return_type = Some(
                    rem.into_inner()
                        .next()
                        .ok_or_else(|| {
                            LoweringError::missing(
                                &item_error,
                                ExpectedSyntax::ExternalBlockReturnType,
                            )
                        })?
                        .as_str()
                        .to_string(),
                );
            }
            _ => {
                return Err(LoweringError::invalid(
                    &rem,
                    ParserIssue::ExternalBlockFunctionChild,
                ));
            }
        }
    }

    Ok(Statement::ExternFunctionDeclaration {
        name: name.ok_or_else(|| {
            LoweringError::missing(&item_error, ExpectedSyntax::ExternalBlockFunctionName)
        })?,
        parameters,
        return_type,
        abi,
        is_pure,
    })
}

fn is_abi_rule(rule: Rule) -> bool {
    matches!(rule, Rule::string | Rule::char_literal | Rule::identifier)
}

fn parse_external_block_parameters(
    pair: Pair<Rule>,
    function: &Pair<Rule>,
) -> LoweringResult<Vec<ExternParameter>> {
    let mut parameters = Vec::new();
    let Some(parameter_list) = pair.into_inner().next() else {
        return Ok(parameters);
    };

    for parameter_item in parameter_list.into_inner() {
        let item_error = parameter_item.clone();
        let parameter = parameter_item.into_inner().next().ok_or_else(|| {
            LoweringError::missing(&item_error, ExpectedSyntax::ExternalBlockParameter)
        })?;

        if parameter.as_rule() == Rule::parameter_self {
            return Err(LoweringError::invalid(
                &parameter,
                ParserIssue::ExternalBlockSelfParameter,
            ));
        }

        let mut name = None;
        let mut type_name = None;

        for part in parameter.into_inner() {
            match part.as_rule() {
                Rule::identifier => name = Some(part.as_str().to_string()),
                Rule::type_specifier => type_name = Some(part.as_str().to_string()),
                _ => {}
            }
        }

        parameters.push(ExternParameter {
            name: name.ok_or_else(|| {
                LoweringError::missing(function, ExpectedSyntax::ExternalBlockParameterName)
            })?,
            type_name: type_name.ok_or_else(|| {
                LoweringError::missing(function, ExpectedSyntax::ExternalBlockParameterType)
            })?,
        });
    }

    Ok(parameters)
}
