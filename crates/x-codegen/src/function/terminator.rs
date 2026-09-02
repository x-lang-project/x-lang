use inkwell::attributes::{Attribute, AttributeLoc};

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::{BlockIdentifier, MirTerminator, Operand, Place};

struct CallContinuation<'a> {
    destination: &'a Place,
    target: &'a BlockIdentifier,
}

impl FunctionCompiler<'_, '_> {
    fn return_type_or_void(&self) -> &Type {
        self.body
            .locals
            .get(&self.body.return_local)
            .map(|declaration| &declaration.value_type)
            .unwrap_or(&Type::Void)
    }
}

// musttail requires matching storage and extension attributes.
const TAIL_ABI_ATTRIBUTES: &[&str] = &[
    "sret",
    "byval",
    "inreg",
    "inalloca",
    "preallocated",
    "byref",
    "returned",
    "signext",
    "zeroext",
    "swiftself",
    "swifterror",
    "swiftasync",
    "nest",
];

fn tail_abi_locations(parameters: u32) -> impl Iterator<Item = AttributeLoc> {
    std::iter::once(AttributeLoc::Return).chain((0..parameters).map(AttributeLoc::Param))
}

fn tail_abi_attributes_match(
    caller: inkwell::values::FunctionValue<'_>,
    callee: inkwell::values::FunctionValue<'_>,
) -> bool {
    caller.count_params() == callee.count_params()
        && tail_abi_locations(caller.count_params()).all(|location| {
            TAIL_ABI_ATTRIBUTES.iter().all(|name| {
                let kind = Attribute::get_named_enum_kind_id(name);

                kind == 0
                    || caller.get_enum_attribute(location, kind)
                        == callee.get_enum_attribute(location, kind)
            })
        })
}
