mod specialization;

use super::*;
use crate::FunctionDefinition;
use crate::MultiFunctionCandidate;
use crate::purity::ImpureOperation;
use crate::symbols::qualified_call_path_matches;
use x_ast::DeclarationKey;
use x_ast::GenericParameter;
use x_ast::SourceError;
use x_ast::intrinsics;
use x_ast::intrinsics::{Intrinsic, LayoutIntrinsic};
use x_diagnostics::frontend_messages::FrontendMessage;
