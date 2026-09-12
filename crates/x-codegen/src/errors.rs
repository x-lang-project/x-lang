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

impl CodeGenerationError {
    pub fn into_diagnostic(self, span: Span) -> Diagnostic {
        match self {
            CodeGenerationError::BuilderError(error) => {
                backend_messages::codegen_builder_error(error, span)
            }
            CodeGenerationError::TypeNotFound(name) => {
                backend_messages::codegen_type_not_found(name, span)
            }
            CodeGenerationError::UnsupportedType(value_type) => {
                backend_messages::codegen_unsupported_type(value_type, span)
            }
            CodeGenerationError::FunctionSignatureMissing(name) => {
                backend_messages::codegen_function_signature_missing(name, span)
            }
            CodeGenerationError::MissingTerminator(block) => {
                backend_messages::codegen_missing_terminator(block, span)
            }
            CodeGenerationError::MissingEntryBlock => {
                backend_messages::codegen_missing_entry_block(span)
            }
            CodeGenerationError::MissingLocalAllocation(identifier) => {
                backend_messages::codegen_missing_local_allocation(identifier, span)
            }
            CodeGenerationError::MissingHardwareArgument(position) => {
                backend_messages::codegen_missing_hardware_argument(position, span)
            }
            CodeGenerationError::UnsupportedOperator { operator } => {
                backend_messages::codegen_unsupported_operator(operator, span)
            }
            CodeGenerationError::UnsupportedUnaryOperator { operator } => {
                backend_messages::codegen_unsupported_unary_operator(operator, span)
            }
            CodeGenerationError::UnsupportedCast { to } => {
                backend_messages::codegen_unsupported_cast(to, span)
            }
            CodeGenerationError::TypeHasNoSize(value_type) => {
                backend_messages::codegen_type_has_no_size(value_type, span)
            }
            CodeGenerationError::MissingCalleeSymbol(name) => {
                backend_messages::codegen_missing_callee_symbol(name, span)
            }
            CodeGenerationError::JitExecutionFailure(name) => {
                backend_messages::codegen_jit_execution_failure(name, span)
            }
            CodeGenerationError::LlvmInternalError(detail) => {
                backend_messages::codegen_llvm_internal_error(detail, span)
            }
            CodeGenerationError::JitInitializationFailure(detail) => {
                backend_messages::codegen_jit_initialization_failure(detail, span)
            }
            CodeGenerationError::MissingMirBlock(block) => {
                backend_messages::codegen_missing_mir_block(block, span)
            }
            CodeGenerationError::MissingLlvmBlock(block) => {
                backend_messages::codegen_missing_llvm_block(block, span)
            }
            CodeGenerationError::LlvmVerificationFailure(detail) => {
                backend_messages::codegen_llvm_verification_failure(detail, span)
            }
            CodeGenerationError::InvalidMir(detail) => {
                backend_messages::codegen_invalid_mir(detail, span)
            }
        }
    }
}

impl Display for CodeGenerationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuilderError(value0) => {
                formatter.write_str(&backend_messages::codegen_builder_error_display(value0))
            }
            Self::TypeNotFound(value0) => {
                formatter.write_str(&backend_messages::codegen_type_not_found_display(value0))
            }
            Self::UnsupportedType(value0) => {
                formatter.write_str(&backend_messages::codegen_unsupported_type_display(value0))
            }
            Self::FunctionSignatureMissing(value0) => formatter
                .write_str(&backend_messages::codegen_function_signature_missing_display(value0)),
            Self::MissingTerminator(value0) => formatter.write_str(
                &backend_messages::codegen_missing_terminator_display(value0),
            ),
            Self::MissingEntryBlock => {
                formatter.write_str(&backend_messages::codegen_missing_entry_block_display())
            }
            Self::MissingLocalAllocation(value0) => formatter.write_str(
                &backend_messages::codegen_missing_local_allocation_display(value0),
            ),
            Self::MissingHardwareArgument(value0) => formatter
                .write_str(&backend_messages::codegen_missing_hardware_argument_display(value0)),
            Self::UnsupportedOperator { operator } => formatter.write_str(
                &backend_messages::codegen_unsupported_operator_display(operator),
            ),
            Self::UnsupportedUnaryOperator { operator } => formatter
                .write_str(&backend_messages::codegen_unsupported_unary_operator_display(operator)),
            Self::UnsupportedCast { to } => {
                formatter.write_str(&backend_messages::codegen_unsupported_cast_display(to))
            }
            Self::TypeHasNoSize(value0) => {
                formatter.write_str(&backend_messages::codegen_type_has_no_size_display(value0))
            }
            Self::MissingCalleeSymbol(value0) => formatter.write_str(
                &backend_messages::codegen_missing_callee_symbol_display(value0),
            ),
            Self::JitExecutionFailure(value0) => formatter.write_str(
                &backend_messages::codegen_jit_execution_failure_display(value0),
            ),
            Self::LlvmInternalError(value0) => formatter.write_str(
                &backend_messages::codegen_llvm_internal_error_display(value0),
            ),
            Self::JitInitializationFailure(value0) => formatter
                .write_str(&backend_messages::codegen_jit_initialization_failure_display(value0)),
            Self::MissingMirBlock(value0) => {
                formatter.write_str(&backend_messages::codegen_missing_mir_block_display(value0))
            }
            Self::MissingLlvmBlock(value0) => formatter.write_str(
                &backend_messages::codegen_missing_llvm_block_display(value0),
            ),
            Self::LlvmVerificationFailure(value0) => formatter
                .write_str(&backend_messages::codegen_llvm_verification_failure_display(value0)),
            Self::InvalidMir(value0) => {
                formatter.write_str(&backend_messages::codegen_invalid_mir_display(value0))
            }
        }
    }
}

impl error::Error for CodeGenerationError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::BuilderError(error) => Some(error),
            _ => None,
        }
    }
}

impl From<BuilderError> for CodeGenerationError {
    fn from(error: BuilderError) -> Self {
        Self::BuilderError(error)
    }
}
