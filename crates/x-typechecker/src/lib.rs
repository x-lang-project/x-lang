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

use function_context::{EffectContext, FunctionContext};
use x_ast::DeclarationKey;
use x_ast::SourceSpan;
use x_ast::StructDefinition;
use x_ast::{Program, Statement, Type};

mod function_context;
mod helpers;

mod purity;
mod specialization_key;

mod static_specialization;
mod structs;
mod symbols;
mod tail_helpers;
mod traits;
mod types;

pub use symbols::{
    FunctionDefinition, FunctionSignature, MultiFunctionCandidate, SymbolTable, TraitInfo,
    TypeAliasDefinition,
};

pub struct TypeChecker {
    pub(crate) symbols: SymbolTable,
    pub(crate) declaration_sources: HashMap<DeclarationKey, SourceSpan>,
    pub(crate) program: Program,
    processed_files: HashSet<String>,
    function_contexts: Vec<FunctionContext>,

    // Scope depth keeps shadowed bindings' provenance separate.
    pub(crate) pure_local_references:
        HashMap<(usize, String), HashSet<Vec<purity::LocalProjection>>>,
    return_types: Vec<Type>,
    pending_monomorphs: Vec<Statement>,
    effects: EffectContext,
    expression_depth: usize,
    type_resolution_depth: usize,
    pub(crate) substitutions: HashMap<usize, Type>,
    next_infer_identifier: usize,
    pub(crate) current_namespace: Option<String>,
}

impl TypeChecker {
    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    pub fn declaration_sources(&self) -> &HashMap<DeclarationKey, SourceSpan> {
        &self.declaration_sources
    }
}
