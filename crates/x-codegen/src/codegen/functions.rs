mod body;
mod locals;
mod signatures;

use inkwell::values::FunctionValue;

use super::CodeGeneration;
use crate::codegen::items::{FunctionDeclaration, MirProgramItems};
use crate::errors::CodeGenerationError;
use x_mir::{MirBody, MirProgram};
