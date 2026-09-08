mod lowering;

use inkwell::IntPredicate;
use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use lowering::{LlvmIntrinsicDefinition, ZeroInput};
use x_ast::Type;
use x_ast::intrinsics::{AlignOperation, NumberOperation};
use x_diagnostics::backend_messages;
use x_mir::Operand;
