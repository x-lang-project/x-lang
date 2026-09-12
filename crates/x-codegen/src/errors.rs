use std::error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use inkwell::builder::BuilderError;
use inkwell::support::LLVMString;

use x_ast::{Operator, Type, UnaryOperator};
use x_diagnostics::backend_messages;
use x_diagnostics::{Diagnostic, Span};

#[derive(Debug)]
pub enum CodeGenerationError {
    BuilderError(BuilderError),
    TypeNotFound(String),
    UnsupportedType(Type),
    FunctionSignatureMissing(String),
    MissingTerminator(usize),
    MissingEntryBlock,
    MissingLocalAllocation(usize),
    MissingHardwareArgument(u32),
    UnsupportedOperator { operator: Operator },
    UnsupportedUnaryOperator { operator: UnaryOperator },
    UnsupportedCast { to: Type },
    TypeHasNoSize(Type),
    MissingCalleeSymbol(String),
    JitExecutionFailure(String),
    LlvmInternalError(String),
    JitInitializationFailure(String),
    MissingMirBlock(usize),
    MissingLlvmBlock(usize),
    LlvmVerificationFailure(String),
    InvalidMir(String),
}

impl From<LLVMString> for CodeGenerationError {
    fn from(error: LLVMString) -> Self {
        CodeGenerationError::LlvmInternalError(error.to_string())
    }
}

impl From<BuilderError> for CodeGenerationError {
    fn from(error: BuilderError) -> Self {
        Self::BuilderError(error)
    }
}
