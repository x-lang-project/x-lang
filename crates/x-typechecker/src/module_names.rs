use std::collections::HashSet;

use x_ast::GenericParameter;
use x_ast::Pattern;
use x_ast::Program;
use x_ast::{
    Expression, ExpressionKind, ImplDefinition, Parameter, Statement, StructDefinition,
    TraitDefinition, Type, TypeVisitorMut,
};

fn qualify_module_names(statements: &mut [Statement], mut qualifier: ModuleQualifier<'_>) {
    for statement in statements {
        qualifier.visit_statement(statement);
    }
}

struct ModuleQualifier<'a> {
    functions: &'a HashSet<String>,
    variables: &'a HashSet<String>,
    types: &'a HashSet<String>,
    prefix: &'a str,
    scopes: Vec<Scope>,
}

#[derive(Default)]
struct Scope {
    values: HashSet<String>,
    types: HashSet<String>,
}

mod scopes;
mod visitor;

pub fn qualify_module(program: &mut Program, prefix: &str) {
    let local_functions: HashSet<String> = program
        .statements
        .iter()
        .filter_map(|statement| match statement {
            Statement::Function { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();
    let local_variables: HashSet<String> = program
        .statements
        .iter()
        .filter_map(|statement| match statement {
            Statement::VariableDeclaration {
                pattern: Pattern::Identifier(name),
                ..
            }
            | Statement::Constant { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();
    let local_types: HashSet<String> = program
        .statements
        .iter()
        .filter_map(|statement| match statement {
            Statement::StructDeclaration(definition) => Some(definition.name.clone()),
            Statement::TraitDeclaration(definition) => Some(definition.name.clone()),
            Statement::TypeAlias { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();

    qualify_module_names(
        &mut program.statements,
        ModuleQualifier {
            functions: &local_functions,
            variables: &local_variables,
            types: &local_types,
            prefix,
            scopes: Vec::new(),
        },
    );

    for statement in &mut program.statements {
        match statement {
            Statement::StructDeclaration(definition) => {
                if !definition.name.contains("::") {
                    definition.name = format!("{prefix}::{}", definition.name);
                }
            }
            Statement::TraitDeclaration(definition) => {
                if !definition.name.contains("::") {
                    definition.name = format!("{prefix}::{}", definition.name);
                }
            }
            Statement::TypeAlias { name, .. } => {
                if !name.contains("::") {
                    *name = format!("{prefix}::{name}");
                }
            }
            Statement::Function { name, .. }
            | Statement::VariableDeclaration {
                pattern: Pattern::Identifier(name),
                ..
            }
            | Statement::Constant { name, .. }
                if !name.contains("::") =>
            {
                *name = format!("{prefix}::{name}");
            }
            _ => {}
        }
    }
}
