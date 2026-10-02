use std::collections::HashMap;
use std::path::{Path, PathBuf};

use x_ast::DeclarationKey;
use x_ast::SourceSpan;
use x_ast::{Expression, Program, Statement, Type, TypeVisitorMut};
use x_diagnostics::CompilationFailure;
use x_diagnostics::contract_messages;

#[derive(Default)]
pub struct SourceCatalog {
    files: Vec<(PathBuf, String)>,
    pub(crate) declarations: HashMap<DeclarationKey, SourceSpan>,
}

impl SourceCatalog {
    pub fn declaration_spans(&self) -> &HashMap<DeclarationKey, SourceSpan> {
        &self.declarations
    }
}

struct SourceMarker(u32);
