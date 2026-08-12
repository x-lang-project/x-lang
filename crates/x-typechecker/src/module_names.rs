use std::collections::HashSet;

use x_ast::GenericParameter;
use x_ast::Pattern;
use x_ast::Program;
use x_ast::{
    Expression, ExpressionKind, ImplDefinition, Parameter, Statement, StructDefinition,
    TraitDefinition, Type, TypeVisitorMut,
};

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
