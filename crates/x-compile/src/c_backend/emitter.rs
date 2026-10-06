use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::fmt::Write;

use super::error::EmitError;

use representation::*;
use x_ast::{Operator, Type, UnaryOperator, intrinsics::BuiltinKind};
use x_diagnostics::backend_messages;
use x_mir::*;

type Result<T> = std::result::Result<T, EmitError>;

mod representation;
