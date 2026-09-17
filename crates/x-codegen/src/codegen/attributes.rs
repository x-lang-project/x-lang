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
    pub(super) fn apply_function_semantic_attributes(
        &self,
        function: FunctionValue<'ctx>,
        parameters: &[Type],
        meta: &FunctionMeta,
    ) {
        // Memoised caches access memory even when the source body is pure.
        if meta.is_memoised {
            self.add_function_string_attribute(function, "xlang.memoise", "true");
            let capacity = match &meta.call_contract {
                Some(CallContract::Memoise(contract)) => contract.capacity,
                None => x_mir::DEFAULT_MEMOISE_CAPACITY,
            };

            self.add_function_string_attribute(
                function,
                "xlang.memoise.capacity",
                &capacity.to_string(),
            );
        }

        if meta.is_pure {
            self.add_function_string_attribute(function, "xlang.pure", "true");

            if !meta.is_memoised && parameters.iter().all(pure_no_memory_parameter) {
                self.add_enum_attribute(
                    function,
                    AttributeLoc::Function,
                    LlvmEnumAttribute::Memory,
                );
                self.add_enum_attribute(
                    function,
                    AttributeLoc::Function,
                    LlvmEnumAttribute::NoUnwind,
                );
            }
        }

        if meta.is_multi {
            self.add_function_string_attribute(function, "xlang.multi", "true");
        }

        if meta.is_static || meta.has_static_parameters {
            self.add_function_string_attribute(function, "xlang.static", "true");
        }

        if meta.is_throws {
            self.add_function_string_attribute(function, "xlang.throws", "true");
        }

        let unique_noalias_proved = unique_noalias_is_proved(parameters, &self.records);

        for (index, parameter) in parameters.iter().enumerate() {
            if matches!(
                parameter,
                Type::Reference {
                    is_unique: true,
                    ..
                }
            ) {
                self.add_parameter_string_attribute(function, index as u32, "xlang.unique", "true");
                self.add_enum_attribute(
                    function,
                    AttributeLoc::Param(index as u32),
                    LlvmEnumAttribute::NonNull,
                );

                if unique_noalias_proved {
                    self.add_enum_attribute(
                        function,
                        AttributeLoc::Param(index as u32),
                        LlvmEnumAttribute::NoAlias,
                    );
                }
            }
        }
    }

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

fn c_integer_extension_attribute(value_type: &Type) -> Option<LlvmEnumAttribute> {
    match value_type {
        Type::I8 | Type::I16 => Some(LlvmEnumAttribute::SignExtend),
        Type::U8 | Type::U16 | Type::Boolean | Type::Character => {
            Some(LlvmEnumAttribute::ZeroExtend)
        }
        _ => None,
    }
}
