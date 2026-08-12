use std::collections::HashSet;

use crate::TypeChecker;
use x_ast::{DeclarationKey, Pattern, Statement, Type};
use x_diagnostics::frontend_messages::FrontendIssue;

mod purity;
