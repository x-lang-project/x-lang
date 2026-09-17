use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::values::FunctionValue;

use super::CodeGeneration;
use memory_contracts::*;
use x_ast::Type;
use x_mir::CallContract;
use x_mir::FunctionMeta;

mod memory_contracts;

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

impl<'ctx> CodeGeneration<'ctx> {
    fn add_enum_attribute(
        &self,
        function: FunctionValue<'ctx>,
        location: AttributeLoc,
        attribute: LlvmEnumAttribute,
    ) {
        let kind = Attribute::get_named_enum_kind_id(attribute.llvm_name());

        if kind == 0 {
            return;
        }

        let attribute = self.context.create_enum_attribute(kind, 0);
        function.add_attribute(location, attribute);
    }

    pub(super) fn add_parameter_string_attribute(
        &self,
        function: FunctionValue<'ctx>,
        index: u32,
        name: &str,
        value: &str,
    ) {
        let Some(parameter) = function.get_nth_param(index) else {
            return;
        };

        if !parameter.is_pointer_value() {
            return;
        }

        let attribute = self.context.create_string_attribute(name, value);
        function.add_attribute(AttributeLoc::Param(index), attribute);
    }

    pub(super) fn add_function_string_attribute(
        &self,
        function: FunctionValue<'ctx>,
        name: &str,
        value: &str,
    ) {
        let attribute = self.context.create_string_attribute(name, value);
        function.add_attribute(AttributeLoc::Function, attribute);
    }
}
