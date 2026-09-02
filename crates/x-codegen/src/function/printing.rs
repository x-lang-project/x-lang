use std::slice;

use inkwell::values;
use inkwell::values::BasicMetadataValueEnum;

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::Operand;

#[derive(Clone, Copy)]
enum ScalarPrintFormat {
    Float,
    Signed32,
    Unsigned32,
    Signed64,
    Unsigned64,
}

impl ScalarPrintFormat {
    const fn format(self, newline: bool) -> &'static str {
        match (self, newline) {
            (Self::Float, false) => "%g",
            (Self::Float, true) => "%g\n",
            (Self::Signed32, false) => "%d",
            (Self::Signed32, true) => "%d\n",
            (Self::Unsigned32, false) => "%u",
            (Self::Unsigned32, true) => "%u\n",
            (Self::Signed64, false) => "%lld",
            (Self::Signed64, true) => "%lld\n",
            (Self::Unsigned64, false) => "%llu",
            (Self::Unsigned64, true) => "%llu\n",
        }
    }
}
