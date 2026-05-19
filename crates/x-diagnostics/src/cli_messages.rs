use std::fmt;

use crate::Error;
use crate::cli;
use crate::{CompilationFailure, OperationalDiagnostic};

pub struct CliDiagnostic;

mod cheri;
