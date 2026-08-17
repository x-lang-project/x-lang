use std::collections::HashMap;

use crate::FunctionDefinition;
use crate::FunctionSignature;
use crate::MultiFunctionCandidate;
use crate::TraitInfo;
use crate::TypeAliasDefinition;
use crate::TypeChecker;

use x_ast::SourceError;
use x_ast::{Pattern, Statement, Type};
use x_diagnostics::frontend_messages::FrontendMessage;

mod declarations;

struct ExternDeclaration<'a> {
    name: &'a str,
    parameters: &'a [x_ast::ExternParameter],
    return_type: Option<&'a str>,
    abi: Option<&'a str>,
    is_pure: bool,
}

impl TypeChecker {
    fn qualify_indexed_name(&self, name: &str) -> String {
        match &self.current_namespace {
            Some(namespace) if !name.contains("::") => format!("{namespace}::{name}"),
            _ => name.to_string(),
        }
    }
}
