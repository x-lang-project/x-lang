use std::collections::HashMap;

use x_ast::{GenericParameter, Parameter, Statement, Type};

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub generic_parameters: Vec<GenericParameter>,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub body: Vec<Statement>,
    pub is_exported: bool,
    pub is_pure: bool,
    pub is_memoised: bool,
    pub is_static: bool,
    pub is_multi: bool,
    pub is_throws: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    pub is_throws: bool,
    pub parameter_types: Vec<Type>,
    pub return_type: Type,
    pub is_pure: bool,
    pub is_static: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiFunctionCandidate {
    pub mangled_name: String,
    pub signature: FunctionSignature,
}

#[derive(Debug, Clone)]
pub struct TraitInfo {
    pub name: String,
    pub methods: HashMap<String, FunctionSignature>,
}

#[derive(Debug, Clone)]
pub struct TypeAliasDefinition {
    pub generic_parameters: Vec<x_ast::GenericParameter>,
    pub value_type: Type,
}
