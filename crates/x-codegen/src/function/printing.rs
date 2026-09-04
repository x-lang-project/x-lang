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
    fn for_type(value_type: &Type) -> Option<Self> {
        Some(match value_type {
            Type::F32 | Type::F64 => Self::Float,
            Type::I8 | Type::I16 | Type::I32 => Self::Signed32,
            Type::U8 | Type::U16 | Type::U32 => Self::Unsigned32,
            Type::I64 | Type::ISize => Self::Signed64,
            Type::U64 | Type::USize => Self::Unsigned64,
            _ => return None,
        })
    }

    fn load_type(self) -> Type {
        match self {
            Self::Float => Type::F64,
            Self::Signed32 => Type::I32,
            Self::Unsigned32 => Type::U32,
            Self::Signed64 => Type::I64,
            Self::Unsigned64 => Type::U64,
        }
    }

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
