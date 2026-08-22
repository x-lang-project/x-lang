mod functions;

use std::collections::HashMap;

use crate::purity::ImpureOperation;
use crate::{TypeChecker, errors::TypeError};
use x_ast::SourceError;
use x_ast::{Expression, ExpressionKind, Parameter, Pattern, Statement, Type};
use x_diagnostics::frontend_messages::ExecutableTypeReason;
use x_diagnostics::frontend_messages::FrontendMessage;
