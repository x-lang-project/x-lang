use inkwell::types;
use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Operator;
use x_ast::Type;
use x_diagnostics::backend_messages;

#[derive(Clone, Copy)]
enum ValueKind {
    Integer,
    Pointer,
    Aggregate,
    Other,
}

#[derive(Clone, Copy)]
enum Coercion {
    Identity,
    IntegerCast { signed: bool },
    PointerCast,
    LoadAggregate,
    SpillAggregate,
}

#[derive(Clone, Copy)]
struct CallArgument<'a> {
    function: &'a str,
    index: usize,
}

impl ValueKind {
    fn of(value: BasicValueEnum<'_>) -> Self {
        Self::of_type(value.get_type())
    }

    fn of_type(value_type: types::BasicTypeEnum<'_>) -> Self {
        if value_type.is_int_type() {
            Self::Integer
        } else if value_type.is_pointer_type() {
            Self::Pointer
        } else if value_type.is_struct_type() || value_type.is_array_type() {
            Self::Aggregate
        } else {
            Self::Other
        }
    }
}

pub(super) struct PointerArithmeticTypes<'a> {
    pub pointee: Option<&'a Type>,
    pub offset: &'a Type,
}
