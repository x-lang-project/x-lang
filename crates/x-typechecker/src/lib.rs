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

pub mod expression;
mod function_context;
mod helpers;
mod indexing;
mod inference;
mod module_names;
mod program;
mod purity;
mod specialization_key;
pub mod statement;
mod static_specialization;
mod structs;
mod symbols;
mod tail_helpers;
mod traits;
mod types;

pub use module_names::qualify_module;
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

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    pub fn checked_program(&self) -> &Program {
        &self.program
    }

    pub fn set_declaration_sources(&mut self, sources: &HashMap<DeclarationKey, SourceSpan>) {
        self.declaration_sources.clone_from(sources);
    }

    pub fn declaration_sources(&self) -> &HashMap<DeclarationKey, SourceSpan> {
        &self.declaration_sources
    }

    pub fn declare_variable(&mut self, name: String, value_type: Type) {
        self.symbols.add_variable(name, value_type);
    }

    pub fn declare_type_alias(&mut self, name: String, definition: TypeAliasDefinition) {
        self.symbols.type_aliases.insert(name, definition);
    }

    pub fn declare_generic_struct(&mut self, name: String, definition: StructDefinition) {
        if let Some(invariant) = &definition.invariant {
            self.register_struct_invariant(name.clone(), invariant);
        }

        self.symbols.records.declare_template(name, definition);
    }

    pub fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            declaration_sources: HashMap::new(),
            program: Program {
                statements: Vec::new(),
            },
            processed_files: HashSet::new(),
            function_contexts: Vec::new(),
            pure_local_references: HashMap::new(),
            return_types: Vec::new(),
            pending_monomorphs: Vec::new(),
            effects: EffectContext::default(),
            expression_depth: 0,
            type_resolution_depth: 0,
            substitutions: HashMap::new(),
            next_infer_identifier: 0,
            current_namespace: None,
        }
    }
}
