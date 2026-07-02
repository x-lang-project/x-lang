use std::collections::BTreeMap;

use crate::{Expression, GenericParameter, Layout, Pattern, StructInvariant, Type};

impl Statement {
    pub fn declaration_key(&self) -> Option<crate::DeclarationKey> {
        use crate::DeclarationKey;

        Some(match self {
            Self::Function {
                name, parameters, ..
            } => DeclarationKey::Function(
                name.clone(),
                parameters
                    .iter()
                    .map(|parameter| parameter.value_type.clone())
                    .collect(),
            ),
            Self::ExternFunctionDeclaration { name, .. } => DeclarationKey::Extern(name.clone()),
            Self::StructDeclaration(definition) => DeclarationKey::Struct(definition.name.clone()),
            Self::TraitDeclaration(definition) => DeclarationKey::Trait(definition.name.clone()),
            Self::ImplDeclaration(definition) => {
                DeclarationKey::Impl(definition.trait_name.clone(), definition.type_name.clone())
            }
            Self::TypeAlias { name, .. } => DeclarationKey::Alias(name.clone()),
            Self::Constant { name, .. } => DeclarationKey::Constant(name.clone()),
            Self::VariableDeclaration { pattern, .. } => DeclarationKey::Binding(match pattern {
                Pattern::Identifier(name) => vec![name.clone()],
                Pattern::Tuple(names)
                | Pattern::Enum {
                    bindings: names, ..
                } => names.clone(),
                Pattern::Wildcard => return None,
            }),
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression {
        expression: Expression,
    },
    VariableDeclaration {
        pattern: Pattern,
        type_annotation: Option<Type>,
        value: Expression,
    },
    Constant {
        name: String,
        type_annotation: Option<Type>,
        value: Expression,
    },
    Import {
        module: String,
        item: String,
    },
    TypeAlias {
        name: String,
        generic_parameters: Vec<GenericParameter>,
        value_type: Type,
    },
    Function {
        name: String,
        generic_parameters: Vec<GenericParameter>,
        parameters: Vec<Parameter>,
        return_type: Type,
        body: Option<Box<Vec<Statement>>>,
        is_pure: bool,
        is_memoised: bool,
        is_static: bool,
        is_multi: bool,
        is_throws: bool,
        is_exported: bool,
    },
    ExternFunctionDeclaration {
        name: String,
        parameters: Vec<ExternParameter>,
        return_type: Option<String>,
        abi: Option<String>,
        is_pure: bool,
    },
    Assert {
        condition: Expression,
        message: Option<Expression>,
    },
    Panic {
        message: Option<Expression>,
    },

    Throw {
        message: Expression,
    },

    Block {
        statements: Vec<Statement>,
    },
    Become {
        call: Expression,
    },
    Comment(String),
    ForRangeLoop {
        variable: String,
        start: Box<Expression>,
        end: Box<Expression>,
        body: Vec<Statement>,
        is_parallel: bool,
    },
    If {
        condition: Box<Expression>,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
    WhileLoop {
        condition: Box<Expression>,
        body: Vec<Statement>,
    },
    ForEachLoop {
        variable: String,
        iterator: Box<Expression>,
        body: Vec<Statement>,
        is_parallel: bool,
    },
    StructDeclaration(StructDefinition),
    TraitDeclaration(TraitDefinition),
    ImplDeclaration(ImplDefinition),
    Return {
        value: Option<Expression>,
    },
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternParameter {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterRole {
    Argument,
    Receiver,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub role: ParameterRole,
    pub name: String,
    pub value_type: Type,
    pub is_static: bool,
    pub is_mut: bool,
}

impl Parameter {
    pub fn is_receiver(&self) -> bool {
        self.role == ParameterRole::Receiver
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDefinition {
    pub name: String,
    pub generic_parameters: Option<Vec<GenericParameter>>,
    pub fields: Vec<(String, Type)>,
    pub invariant: Option<StructInvariant>,
    pub layout: Layout,
    pub derives: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructInitialisation {
    pub name: String,
    pub fields: Vec<(String, Expression)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDefinition {
    pub supertraits: Vec<Type>,
    pub name: String,
    pub generic_parameters: Vec<GenericParameter>,
    pub associated_types: BTreeMap<String, Type>,
    pub methods: BTreeMap<String, Option<Statement>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplDefinition {
    pub trait_name: String,
    pub generic_parameters: Vec<GenericParameter>,
    pub type_name: Type,
    pub associated_types: BTreeMap<String, Type>,
    pub methods: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}
