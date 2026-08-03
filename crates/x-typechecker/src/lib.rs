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

use std::collections::{HashMap, HashSet};

use function_context::FunctionContext;
use x_ast::DeclarationKey;
use x_ast::SourceSpan;
use x_ast::StructDefinition;
use x_ast::{Program, Statement, Type};

mod function_context;

mod purity;

mod symbols;

pub use symbols::{
    FunctionDefinition, FunctionSignature, MultiFunctionCandidate, SymbolTable, TraitInfo,
    TypeAliasDefinition,
};
