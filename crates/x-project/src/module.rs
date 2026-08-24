use std::collections::{HashMap, HashSet};
use std::iter;
use std::path::{Path, PathBuf};

use crate::source;

use x_ast::Statement;
use x_diagnostics::CompilationFailure;
use x_diagnostics::ProjectError;
use x_diagnostics::contract_messages;

#[derive(Debug, Clone)]
pub struct ModuleNode {
    pub name: String,
    pub path: PathBuf,
    pub source: String,
    pub submodules: Vec<String>,
    pub imports: Vec<Vec<String>>,
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
}

mod discovery;
mod ordering;
