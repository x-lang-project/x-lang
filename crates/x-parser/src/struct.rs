use pest::iterators::Pair;

use crate::attributes::{self, StructAttribute};
use crate::block;
use crate::{
    Rule,
    expression::parse_expression,
    lowering_error::{LoweringError, LoweringResult},
    types::parse_type,
};
use x_ast::{
    Expression, ExpressionKind, GenericParameter, Layout, Statement, StructDefinition,
    StructInitialisation, Type,
};
use x_diagnostics::parser_messages::{ExpectedSyntax, ParserIssue};

pub fn parse_struct_declaration(pair: Pair<Rule>) -> LoweringResult<Statement> {
    let error_pair = pair.clone();
    let mut layout = None;
    let mut name: Option<String> = None;
    let mut generic_parameters: Option<Vec<GenericParameter>> = None;
    let mut fields: Vec<(String, Type)> = Vec::new();
    let mut invariant: Option<x_ast::StructInvariant> = None;
    let mut derives = Vec::new();

    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::attribute => {
                let attribute = attributes::struct_attribute(child.as_str())
                    .map_err(|detail| LoweringError::invalid(&child, detail))?;

                match attribute {
                    StructAttribute::Layout(value) => {
                        if layout.replace(value).is_some() {
                            return Err(LoweringError::invalid(
                                &child,
                                ParserIssue::DuplicateLayout,
                            ));
                        }
                    }
                    StructAttribute::Derive(names) => derives.extend(names),
                }
            }

            Rule::identifier => {
                if name.is_none() {
                    name = Some(child.as_str().to_string());
                }
            }

            Rule::generic_parameter_definition_list => {
                generic_parameters = Some(crate::statement::generics::parse_generic_parameters(
                    child,
                    crate::statement::generics::LifetimeSpelling::Quoted,
                )?);
            }

            Rule::struct_body => {
                parse_struct_body(child, &mut fields, &mut invariant)?;
            }

            _ => {}
        }
    }

    Ok(Statement::StructDeclaration(StructDefinition {
        name: name
            .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::StructName))?,
        generic_parameters,
        fields,
        invariant,
        layout: layout.unwrap_or(Layout::AoS),
        derives,
    }))
}

fn parse_struct_body(
    body: Pair<Rule>,
    fields: &mut Vec<(String, Type)>,
    invariant: &mut Option<x_ast::StructInvariant>,
) -> LoweringResult<()> {
    for item in body.into_inner() {
        match item.as_rule() {
            Rule::struct_field => {
                fields.push(parse_struct_field(item)?);
            }

            Rule::invariant_block => {
                if invariant.is_some() {
                    return Err(LoweringError::invalid(
                        &item,
                        ParserIssue::DuplicateStructInvariant,
                    ));
                }

                *invariant = parse_invariant(item)?;
            }

            _ => {}
        }
    }

    Ok(())
}

fn parse_struct_field(field_pair: Pair<Rule>) -> LoweringResult<(String, Type)> {
    let error_pair = field_pair.clone();
    let mut name: Option<String> = None;
    let mut value_type: Option<Type> = None;

    for child in field_pair.into_inner() {
        match child.as_rule() {
            Rule::identifier => {
                name = Some(child.as_str().to_string());
            }

            Rule::type_specifier | Rule::r#type => {
                value_type = Some(parse_type(child)?);
            }

            _ => {}
        }
    }

    Ok((
        name.ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::FieldName))?,
        value_type.ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::FieldType))?,
    ))
}

fn parse_invariant(pair: Pair<Rule>) -> LoweringResult<Option<x_ast::StructInvariant>> {
    let error_pair = pair.clone();
    let block_pair = pair
        .into_inner()
        .find(|child| child.as_rule() == Rule::block)
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::InvariantBlock))?;
    let statements = block::parse_block_statements(block_pair)?;

    match statements.as_slice() {
        [Statement::Expression { expression }]
        | [
            Statement::Return {
                value: Some(expression),
            },
        ] => Ok(Some(x_ast::StructInvariant::source(expression.clone()))),
        _ => Err(LoweringError::invalid(
            &error_pair,
            ParserIssue::InvalidStructInvariant,
        )),
    }
}

pub fn parse_struct_instantiate(pair: Pair<Rule>) -> LoweringResult<Expression> {
    let pair_span = crate::source_span_from_pair(&pair);
    let error_pair = pair.clone();
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .ok_or_else(|| LoweringError::missing(&error_pair, ExpectedSyntax::StructName))?
        .as_str()
        .to_string();

    let fields = if let Some(fields_pair) = inner.next() {
        fields_pair
            .into_inner()
            .map(|field_pair| -> LoweringResult<_> {
                let field_error = field_pair.clone();
                let mut field_inner = field_pair.into_inner();
                let field_name = field_inner
                    .next()
                    .ok_or_else(|| LoweringError::missing(&field_error, ExpectedSyntax::FieldName))?
                    .as_str()
                    .to_string();

                let value = match field_inner.next() {
                    Some(value) => parse_expression(value)?,
                    None => Expression {
                        span: None,
                        value_type: Type::Unknown,
                        kind: ExpressionKind::Identifier(field_name.clone()),
                    },
                };

                Ok((field_name, value))
            })
            .collect::<LoweringResult<Vec<_>>>()?
    } else {
        Vec::new()
    };

    Ok(Expression {
        span: Some(pair_span),
        value_type: Type::Unknown,
        kind: ExpressionKind::StructInstantiate(StructInitialisation { name, fields }),
    })
}
