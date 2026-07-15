#![cfg_attr(
    not(test),
    deny(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented,
        clippy::unreachable,
        clippy::unwrap_used
    )
)]

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::slice;

use x_ast::DeclarationKey;
use x_ast::Expression;
use x_ast::ExternParameter;
use x_ast::LocatedError;
use x_ast::SourceSpan;
use x_ast::Statement;
use x_ast::{GenericParameter, Parameter, Pattern, StructDefinition, TraitDefinition, Type};
use x_diagnostics::frontend_messages;

mod lower;
pub use lower::{lower_program, lower_program_with_sources};

mod validate;
pub use validate::HirValidationError;

#[derive(Debug, Clone, PartialEq)]
pub struct HirProgram {
    pub items: Vec<HirItem>,
    pub declaration_spans: HashMap<DeclarationKey, SourceSpan>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirItem {
    Function(HirFunction),
    ExternFunction(HirExternFunction),
    Struct(StructDefinition),
    Trait(TraitDefinition),
    Impl(HirImpl),
    TypeAlias {
        name: String,
        generic_parameters: Vec<GenericParameter>,
        value_type: Type,
    },
    GlobalLet(HirLet),
    Const {
        name: String,
        value_type: Option<Type>,
        value: HirExpression,
    },
    Import {
        module: String,
        item: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirFunction {
    pub name: String,
    pub generic_parameters: Vec<GenericParameter>,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub is_exported: bool,
    pub flags: HirFunctionFlags,
    pub body: Option<HirBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HirFunctionFlags {
    pub pure: bool,
    pub memoised: bool,
    pub is_static: bool,
    pub multi: bool,
    pub throws: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirExternFunction {
    pub name: String,
    pub parameters: Vec<ExternParameter>,
    pub return_type: Option<String>,
    pub abi: Option<String>,
    pub is_pure: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirImpl {
    pub generic_parameters: Vec<GenericParameter>,
    pub trait_name: String,
    pub type_name: Type,
    pub associated_types: BTreeMap<String, Type>,
    pub methods: Vec<HirFunction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirLet {
    pub pattern: Pattern,
    pub type_annotation: Option<Type>,
    pub value: HirExpression,
}

pub type HirBlock = Vec<Statement>;
pub type HirExpression = Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirLowerError {
    Located(LocatedError<HirLowerError>),
    NonFunctionImplItem(String),
    UnsupportedStatement(String),
}

fn type_contains_codegen_parameter(value_type: &Type) -> bool {
    value_type.any_nested_type(|inner| matches!(inner, Type::TypeParameter(_)))
}

impl HirProgram {
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn functions(&self) -> impl Iterator<Item = &HirFunction> {
        self.items.iter().flat_map(|item| match item {
            HirItem::Function(function) => slice::from_ref(function).iter(),
            HirItem::Impl(impl_definition) => impl_definition.methods.iter(),
            _ => [].iter(),
        })
    }

    pub fn function(&self, name: &str) -> Option<&HirFunction> {
        self.functions().find(|function| function.name == name)
    }
}

x_ast::impl_source_error!(HirLowerError);

impl Display for HirLowerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Located(location) => Display::fmt(&location.error, formatter),
            Self::NonFunctionImplItem(item) => Display::fmt(
                &frontend_messages::HirLowerDiagnostic::NonFunctionImplItem(item.clone()),
                formatter,
            ),
            Self::UnsupportedStatement(statement) => Display::fmt(
                &frontend_messages::HirLowerDiagnostic::UnsupportedStatement(statement.clone()),
                formatter,
            ),
        }
    }
}

impl error::Error for HirLowerError {}
