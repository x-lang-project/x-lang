mod lowering;

use super::*;
use lowering::CheckedMethod;
use x_ast::SourceError;
use x_ast::intrinsics::PointerOperation;
use x_diagnostics::frontend_messages::FrontendMessage;

mod pointer;
