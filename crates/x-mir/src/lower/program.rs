mod functions;
mod globals;
mod reachability;

use super::FunctionInput;
use super::MirLowerer;
use crate::MirProgram;
use crate::errors::MirLowerError;
use functions::has_codegen_generic_parameters;
use functions::has_codegen_generic_signature;
use functions::impl_method_owner_name;
use functions::impl_owner_has_erased_generic_arguments;
use functions::lower_hir_function;
use globals::lower_global_item;
use reachability::{compute_reachable_functions, is_reachable_source_function};
use x_ast::{Expression, Pattern, SourceError, Type};
use x_diagnostics::mir_messages;
use x_hir::{HirItem, HirLet, HirProgram};
