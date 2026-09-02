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

pub fn collect_symbol_table(program: &MirProgram, config: MangleConfig) -> SymbolTable {
    let mut symbols = Vec::new();

    for item in &program.items {
        match item {
            GlobalItem::Function {
                name,
                parameter_types,
                return_type,
                body,
                meta,
                ..
            } => {
                let should_mangle = config.enabled
                    && body.is_some()
                    && !meta.is_exported
                    && name != x_ast::ENTRY_POINT_NAME;
                symbols.push(GlobalSymbol {
                    source_name: name.clone(),
                    symbol_name: if should_mangle {
                        mangle_function_name(name, parameter_types, return_type)
                    } else {
                        sanitize_symbol_segment(name)
                    },
                    kind: SymbolKind::Function,
                    parameters: parameter_types.clone(),
                    return_type: Some(return_type.clone()),
                    is_defined: body.is_some(),
                    is_exported: config.export_global_symbols
                        || meta.is_exported
                        || name == x_ast::ENTRY_POINT_NAME,
                });
            }
            GlobalItem::Static {
                name, value_type, ..
            } => {
                let should_mangle = config.enabled;
                symbols.push(GlobalSymbol {
                    source_name: name.clone(),
                    symbol_name: if should_mangle {
                        format!("_XG{}", x_ast::symbol_name_component(name))
                    } else {
                        sanitize_symbol_segment(name)
                    },
                    kind: SymbolKind::Static,
                    parameters: Vec::new(),
                    return_type: Some(value_type.clone()),
                    is_defined: true,
                    is_exported: config.export_global_symbols,
                });
            }
            _ => {}
        }
    }

    SymbolTable { symbols }
}

pub fn mangle_function_name(name: &str, parameters: &[Type], return_value: &Type) -> String {
    let mut output = format!("_XF{}", x_ast::symbol_name_component(name));
    output.push('P');
    output.push_str(&parameters.len().to_string());
    for parameter in parameters {
        output.push('_');
        output.push_str(&mangle_type(parameter));
    }

    output.push('R');
    output.push_str(&mangle_type(return_value));

    output
}

pub use x_ast::type_symbol_component as mangle_type;
