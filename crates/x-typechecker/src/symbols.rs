mod declarations;
mod definitions;
mod function_resolution;
mod mangling;
mod records;
mod scopes;

pub use definitions::{
    FunctionDefinition, FunctionSignature, MultiFunctionCandidate, TraitInfo, TypeAliasDefinition,
};
pub(crate) use function_resolution::qualified_call_path_matches;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use records::RecordCatalogue;
use x_ast::{TraitDefinition as AstTraitDef, Type};

#[derive(Debug)]
pub struct SymbolTable {
    pub(crate) variables: Vec<HashMap<String, Type>>,
    pub(crate) reserved_source_names: HashSet<String>,
    pub(crate) qualified_function_cache: RwLock<HashMap<String, Option<String>>>,
    pub(crate) functions: HashMap<String, FunctionSignature>,
    pub(crate) function_static_parameters: HashMap<String, Vec<bool>>,
    pub(crate) records: RecordCatalogue,
    pub(crate) invariant_structs: HashSet<String>,
    // None marks dependencies that cannot be bounded to fields.
    pub(crate) invariant_dependencies: HashMap<String, Option<HashSet<Vec<String>>>>,
    pub(crate) struct_methods: HashMap<String, HashMap<String, FunctionSignature>>,
    pub(crate) generic_functions: HashMap<String, Arc<FunctionDefinition>>,
    pub(crate) function_instantiations: HashMap<(String, Vec<Type>), String>,
    pub(crate) traits: HashMap<String, TraitInfo>,
    pub(crate) impls_for_type: HashMap<Type, Vec<String>>,
    pub(crate) trait_ast_definitions: HashMap<String, AstTraitDef>,
    pub(crate) multi_functions: HashMap<String, Vec<MultiFunctionCandidate>>,
    pub(crate) imported_aliases: HashMap<String, String>,
    pub(crate) qualified_imports: HashSet<String>,
    pub(crate) type_aliases: HashMap<String, TypeAliasDefinition>,
    pub(crate) global_constants: HashSet<String>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self {
            variables: vec![HashMap::new()],
            reserved_source_names: Default::default(),
            qualified_function_cache: RwLock::new(HashMap::new()),
            functions: Default::default(),
            function_static_parameters: Default::default(),
            records: Default::default(),
            invariant_structs: Default::default(),
            invariant_dependencies: Default::default(),
            struct_methods: Default::default(),
            generic_functions: Default::default(),
            function_instantiations: Default::default(),
            traits: Default::default(),
            impls_for_type: Default::default(),
            trait_ast_definitions: Default::default(),
            multi_functions: Default::default(),
            imported_aliases: Default::default(),
            qualified_imports: Default::default(),
            type_aliases: Default::default(),
            global_constants: Default::default(),
        }
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_primitive_str(type_name: &str) -> Type {
        crate::primitive_type!(type_name)
    }
}
