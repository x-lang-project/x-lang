use std::collections::HashSet;

use self::source::collect_source_namespace_names;
use crate::{TypeChecker, errors::TypeError};
use x_ast::Program;
use x_diagnostics::frontend_messages::FrontendIssue;

mod imports;
mod normalization;
mod source;
