use pest::iterators::Pair;

use crate::{
    Rule,
    lowering_error::{LoweringError, LoweringResult},
    types::{parse_generic_bound_list, parse_type},
};
use x_ast::{GenericParameter, Type};
use x_diagnostics::parser_messages::ExpectedSyntax;

#[derive(Clone, Copy)]
pub(crate) enum LifetimeSpelling {
    Bare,
    Quoted,
}

pub(crate) fn parse_generic_parameters(
    pair: Pair<Rule>,
    spelling: LifetimeSpelling,
) -> LoweringResult<Vec<GenericParameter>> {
    fn collect(
        pair: Pair<Rule>,
        output: &mut Vec<GenericParameter>,
        spelling: LifetimeSpelling,
    ) -> LoweringResult<()> {
        let error_pair = pair.clone();

        match pair.as_rule() {
            Rule::generic_parameter => {
                let mut children = pair.into_inner();
                let Some(first) = children.next() else {
                    return Err(LoweringError::missing(
                        &error_pair,
                        ExpectedSyntax::GenericParameter,
                    ));
                };

                if first.as_rule() == Rule::identifier {
                    let name = first.as_str().to_string();
                    let bounds = children
                        .clone()
                        .find(|child| child.as_rule() == Rule::generic_bound_list)
                        .map(parse_generic_bound_list)
                        .transpose()?
                        .unwrap_or_default();
                    let default = children
                        .rfind(|child| {
                            matches!(child.as_rule(), Rule::type_specifier | Rule::r#type)
                        })
                        .map(parse_type)
                        .transpose()?;
                    output.push(GenericParameter::Type {
                        name,
                        bounds,
                        default,
                    });
                } else {
                    collect(first, output, spelling)?;
                }
            }
            Rule::const_generic_parameter => {
                let mut inner = pair.into_inner();
                let Some(name) = inner
                    .next()
                    .map(|current_pair| current_pair.as_str().to_string())
                else {
                    return Err(LoweringError::missing(
                        &error_pair,
                        ExpectedSyntax::ConstGenericName,
                    ));
                };

                let value_type = inner
                    .find(|current_pair| {
                        matches!(current_pair.as_rule(), Rule::type_specifier | Rule::r#type)
                    })
                    .map(parse_type)
                    .transpose()?
                    .unwrap_or(Type::USize);
                output.push(GenericParameter::Const {
                    name,
                    value_type,
                    default: None,
                });
            }
            Rule::lifetime => output.push(GenericParameter::Lifetime {
                name: match spelling {
                    LifetimeSpelling::Bare => pair.as_str().trim_start_matches('\'').to_string(),
                    LifetimeSpelling::Quoted => pair.as_str().to_string(),
                },
                bounds: Vec::new(),
            }),
            _ => {
                for child in pair.into_inner() {
                    collect(child, output, spelling)?;
                }
            }
        }

        Ok(())
    }

    let mut parameters = Vec::new();
    collect(pair, &mut parameters, spelling)?;

    Ok(parameters)
}
