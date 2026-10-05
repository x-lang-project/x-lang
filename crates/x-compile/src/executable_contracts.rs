use std::collections::HashMap;
use std::slice;

use functions::*;
use x_ast::{Expression, ExpressionKind, Program, Statement, Type};
use x_diagnostics::CompilationFailure;
use x_diagnostics::contract_messages;

mod functions;
