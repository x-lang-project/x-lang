use std::collections::BTreeMap;

use x_ast::Type;
use x_mir::{GlobalItem, MirProgram};

pub use x_utils::sanitize_symbol as sanitize_symbol_segment;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Static,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalSymbol {
    pub source_name: String,
    pub symbol_name: String,
    pub kind: SymbolKind,
    pub parameters: Vec<Type>,
    pub return_type: Option<Type>,
    pub is_defined: bool,
    pub is_exported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SymbolTable {
    pub symbols: Vec<GlobalSymbol>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MangleConfig {
    pub enabled: bool,
    pub export_global_symbols: bool,
}

impl SymbolTable {
    pub fn function_map(&self) -> BTreeMap<String, String> {
        self.map_for_kind(SymbolKind::Function)
    }

    pub fn static_map(&self) -> BTreeMap<String, String> {
        self.map_for_kind(SymbolKind::Static)
    }

    fn map_for_kind(&self, kind: SymbolKind) -> BTreeMap<String, String> {
        self.symbols
            .iter()
            .filter(|symbol| symbol.kind == kind)
            .map(|symbol| (symbol.source_name.clone(), symbol.symbol_name.clone()))
            .collect()
    }
}

pub use x_ast::type_symbol_component as mangle_type;
