mod resolution;

mod identity;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::iter;

use crate::{TypeChecker, helpers::split_top_level_commas};
use identity::*;
use x_ast::SourceError;
use x_ast::TraitDefinition;
use x_ast::{Statement, Type};
use x_ast::{substitute_self_in_statement, substitute_self_in_type};
use x_diagnostics::frontend_messages::FrontendMessage;

#[derive(PartialEq, Eq)]
struct ResolvedTraitInstance {
    key: String,
    arguments: BTreeMap<String, Type>,
}
