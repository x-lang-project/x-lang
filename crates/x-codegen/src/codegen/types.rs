mod layout;

use inkwell::types;
use inkwell::types::{BasicType, BasicTypeEnum};

use super::CodeGeneration;
use crate::errors::CodeGenerationError;
use x_ast::Type;
