mod functions;

mod reachability;

use super::MirLowerer;
use crate::MirProgram;
use crate::errors::MirLowerError;
use functions::impl_method_owner_name;

use reachability::{compute_reachable_functions, is_reachable_source_function};
use x_ast::{Expression, Pattern, SourceError, Type};
use x_diagnostics::mir_messages;
use x_hir::{HirItem, HirLet, HirProgram};
