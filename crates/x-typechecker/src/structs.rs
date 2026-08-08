mod invariants;

use std::collections::HashMap;

use crate::TypeChecker;
use crate::tail_helpers::qualified_path_suffix_matches;
use x_ast::DeclarationKey;
use x_ast::{Statement, StructDefinition, Type};
use x_diagnostics::frontend_messages::FrontendIssue;
