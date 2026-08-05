mod resolution;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::iter;

use crate::TypeChecker;

use x_ast::SourceError;
use x_ast::TraitDefinition;
use x_ast::{Statement, Type};
use x_ast::{substitute_self_in_statement, substitute_self_in_type};
use x_diagnostics::frontend_messages::FrontendMessage;
