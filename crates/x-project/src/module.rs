use std::collections::{HashMap, HashSet};
use std::iter;
use std::path::{Path, PathBuf};

use crate::source;

use x_ast::Statement;
use x_diagnostics::CompilationFailure;
use x_diagnostics::ProjectError;
use x_diagnostics::contract_messages;

const MAX_PROJECT_SOURCE_BYTES: usize = 64 * 1024 * 1024;
const MAX_PROJECT_MODULES: usize = 1_024;
const MAX_MODULE_DEPTH: usize = 64;

#[derive(Debug, Clone)]
pub struct ModuleNode {
    pub name: String,
    pub path: PathBuf,
    pub source: String,
    pub submodules: Vec<String>,
    pub imports: Vec<Vec<String>>,
}

impl ModuleNode {
    pub fn parse_for_compilation(&self) -> Result<x_parser::ParsedModule, CompilationFailure> {
        x_parser::parse_module_with_spans(&self.source, x_parser::ParseOptions::default()).map_err(
            |error| {
                error
                    .with_source(&self.source, &self.path)
                    .with_note(contract_messages::compiling_module_note(&self.name))
            },
        )
    }
}

pub struct ModuleResolver {
    root_directory: PathBuf,
    modules: HashMap<String, ModuleNode>,
    order: Vec<String>,
    total_source_bytes: usize,
    retain_parsed_modules: bool,
    parsed_modules: HashMap<String, x_parser::ParsedModule>,
}

#[derive(Default)]
struct SortTraversal {
    visited: HashSet<String>,
    active: HashSet<String>,
    order: Vec<String>,
}

impl ModuleResolver {
    pub fn root_directory(&self) -> &Path {
        &self.root_directory
    }

    pub fn modules(&self) -> &HashMap<String, ModuleNode> {
        &self.modules
    }

    pub fn order(&self) -> &[String] {
        &self.order
    }

    fn with_parse_cache(entry_path: &Path, retain_parsed_modules: bool) -> Self {
        let root_directory = entry_path.parent().unwrap_or(Path::new(".")).to_path_buf();

        Self {
            root_directory,
            modules: HashMap::new(),
            order: Vec::new(),
            total_source_bytes: 0,
            retain_parsed_modules,
            parsed_modules: HashMap::new(),
        }
    }

    /// Discovery errors leave the graph, byte count and order unchanged.
    pub fn resolve_tree(
        &mut self,
        current_name: &str,
        current_path: &Path,
    ) -> Result<(), CompilationFailure> {
        let discovery = self.discover_tree(current_name, current_path)?;

        if !discovery.modules.is_empty() {
            self.modules.extend(discovery.modules);
            self.parsed_modules.extend(discovery.parsed_modules);
            self.total_source_bytes = discovery.total_source_bytes;
            self.order.clear();
        }

        Ok(())
    }
}

mod discovery;
mod ordering;
