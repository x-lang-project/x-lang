use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::values::FunctionValue;

use super::CodeGeneration;

use x_ast::Type;
use x_mir::CallContract;
use x_mir::FunctionMeta;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LlvmEnumAttribute {
    Memory,
    NoUnwind,
    NonNull,
    NoAlias,
    SignExtend,
    ZeroExtend,
}

impl LlvmEnumAttribute {
    const fn llvm_name(self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::NoUnwind => "nounwind",
            Self::NonNull => "nonnull",
            Self::NoAlias => "noalias",
            Self::SignExtend => "signext",
            Self::ZeroExtend => "zeroext",
        }
    }
}
