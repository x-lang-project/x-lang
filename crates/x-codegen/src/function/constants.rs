mod arrays;
use std::collections::HashMap;

use inkwell::types;
use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::{LocalIdentifier, MirStatement, Operand, Place, Rvalue};
