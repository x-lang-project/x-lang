use std::collections::BTreeMap;

use crate::{HirBlock, HirExpression, HirFunction, HirItem, HirLet, HirProgram};
use types::*;
use x_ast::ExpressionVisitor;
use x_ast::Type;

mod errors;
pub use errors::HirValidationError;

impl HirProgram {
    pub fn validate(&self) -> Result<(), HirValidationError> {
        for (index, item) in self.items.iter().enumerate() {
            let path = format!("item[{index}]");

            match item {
                HirItem::Function(function) => validate_function(function, &path)?,
                HirItem::ExternFunction(function) => {
                    validate_name(&function.name, || format!("{path}.extern.name"))?;
                }
                HirItem::Struct(definition) => {
                    validate_name(&definition.name, || format!("{path}.struct.name"))?;
                    for (field_index, (name, value_type)) in definition.fields.iter().enumerate() {
                        validate_name(name, || format!("{path}.struct.field[{field_index}].name"))?;
                        validate_type(value_type, || {
                            format!("{path}.struct.field[{field_index}].type")
                        })?;
                    }

                    if let Some(invariant) = &definition.invariant {
                        validate_type(&invariant.predicate.value_type, || {
                            format!("{path}.struct.invariant.type")
                        })?;
                    }
                }
                HirItem::Trait(definition) => {
                    validate_name(&definition.name, || format!("{path}.trait.name"))?;
                    validate_associated_types(
                        &definition.associated_types,
                        &format!("{path}.trait"),
                    )?;
                }
                HirItem::Impl(definition) => {
                    validate_type(&definition.type_name, || format!("{path}.impl.type"))?;
                    validate_associated_types(
                        &definition.associated_types,
                        &format!("{path}.impl"),
                    )?;

                    for (method_index, method) in definition.methods.iter().enumerate() {
                        validate_function(method, &format!("{path}.impl.method[{method_index}]"))?;
                    }
                }
                HirItem::TypeAlias {
                    name, value_type, ..
                } => {
                    validate_name(name, || format!("{path}.alias.name"))?;
                    validate_type(value_type, || format!("{path}.alias.type"))?;
                }
                HirItem::GlobalLet(binding) => validate_let(binding, &path)?,
                HirItem::Const {
                    name,
                    value_type,
                    value,
                } => validate_const(name, value_type, value, &path)?,
                HirItem::Import { module, item } => {
                    validate_name(module, || format!("{path}.import.module"))?;
                    validate_name(item, || format!("{path}.import.item"))?;
                }
            }
        }

        Ok(())
    }
}

fn validate_associated_types(
    types: &BTreeMap<String, Type>,
    path: &str,
) -> Result<(), HirValidationError> {
    for (name, value_type) in types {
        validate_name(name, || format!("{path}.associated_type.name"))?;
        validate_type(value_type, || format!("{path}.associated_type[{name}]"))?;
    }

    Ok(())
}

fn validate_function(function: &HirFunction, path: &str) -> Result<(), HirValidationError> {
    validate_name(&function.name, || format!("{path}.function.name"))?;
    for (index, parameter) in function.parameters.iter().enumerate() {
        validate_name(&parameter.name, || format!("{path}.param[{index}].name"))?;
        validate_type(&parameter.value_type, || {
            format!("{path}.param[{index}].type")
        })?;
    }

    validate_type(&function.return_type, || format!("{path}.return_type"))?;
    if let Some(body) = &function.body {
        validate_block(body, &format!("{path}.body"))?;
    }

    Ok(())
}

fn validate_block(block: &HirBlock, path: &str) -> Result<(), HirValidationError> {
    for (index, statement) in block.iter().enumerate() {
        validate_statement(statement, &format!("{path}.statement[{index}]"))?;
    }

    Ok(())
}

fn validate_let(binding: &HirLet, path: &str) -> Result<(), HirValidationError> {
    if let Some(value_type) = &binding.type_annotation {
        validate_type(value_type, || format!("{path}.let.type"))?;
    }

    validate_expression(&binding.value, &format!("{path}.let.value"))
}

fn validate_const(
    name: &str,
    value_type: &Option<Type>,
    value: &HirExpression,
    path: &str,
) -> Result<(), HirValidationError> {
    validate_name(name, || format!("{path}.const.name"))?;
    if let Some(value_type) = value_type {
        validate_type(value_type, || format!("{path}.const.type"))?;
    }

    validate_expression(value, &format!("{path}.const.value"))
}

fn validate_statement(statement: &x_ast::Statement, path: &str) -> Result<(), HirValidationError> {
    use x_ast::Statement;

    match statement {
        Statement::VariableDeclaration {
            type_annotation, ..
        }
        | Statement::Constant {
            type_annotation, ..
        } => {
            if let Some(value_type) = type_annotation {
                validate_type(value_type, || format!("{path}.binding.type"))?;
            }
        }
        _ => {}
    }

    let mut visitor = Validator {
        path,
        result: Ok(()),
    };

    x_ast::walk_statement_expressions(&mut visitor, statement);

    visitor.result
}

struct Validator<'a> {
    path: &'a str,
    result: Result<(), HirValidationError>,
}
impl ExpressionVisitor for Validator<'_> {
    fn visit_statement(&mut self, statement: &x_ast::Statement) {
        if self.result.is_ok() {
            self.result = validate_statement(statement, self.path);
        }
    }

    fn visit_expression(&mut self, expression: &HirExpression) {
        if self.result.is_ok() {
            self.result = validate_expression(expression, self.path);
        }
    }
}

fn validate_expression(expression: &HirExpression, path: &str) -> Result<(), HirValidationError> {
    use x_ast::ExpressionKind;

    validate_type(&expression.value_type, || format!("{path}.type"))?;
    match &expression.kind {
        ExpressionKind::Float(value) if !value.is_finite() => {
            return Err(HirValidationError::NonFiniteFloat { path: path.into() });
        }
        ExpressionKind::Identifier(name) => validate_name(name, || format!("{path}.local"))?,
        ExpressionKind::FunctionCall { target: name, .. } => {
            types::validate_call_target(name, &format!("{path}.callee"))?
        }
        ExpressionKind::MethodCall { target: method, .. } => {
            types::validate_call_target(method, &format!("{path}.method"))?
        }
        ExpressionKind::FieldAccess { field, .. } => {
            validate_name(field, || format!("{path}.field"))?
        }
        ExpressionKind::StructInstantiate(initialise) => {
            validate_name(&initialise.name, || format!("{path}.struct.name"))?;

            for (name, _) in &initialise.fields {
                validate_name(name, || format!("{path}.field.name"))?;
            }
        }
        ExpressionKind::Cast { to, .. }
        | ExpressionKind::TypeLiteral(to)
        | ExpressionKind::SizeOf(to) => validate_type(to, || format!("{path}.embedded_type"))?,
        ExpressionKind::AnonymousFunction {
            parameters,
            return_type,
            ..
        } => {
            for parameter in parameters {
                validate_name(parameter, || format!("{path}.closure.param"))?;
            }

            if let Some(value_type) = return_type {
                validate_type(value_type, || format!("{path}.closure.return_type"))?;
            }
        }
        _ => {}
    }

    let mut visitor = Validator {
        path,
        result: Ok(()),
    };

    x_ast::walk_expression_children(&mut visitor, expression);

    visitor.result
}

mod types;
