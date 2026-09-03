use inkwell::values::BasicValueEnum;
use inkwell::{AddressSpace, targets, values};

use super::FunctionCompiler;
use crate::{codegen, errors::CodeGenerationError};
use x_ast::{ReferenceQualifiers, Type};
use x_diagnostics::backend_messages;
use x_mir::{Operand, Place};

fn sanitize_trace_field(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], "_")
}
