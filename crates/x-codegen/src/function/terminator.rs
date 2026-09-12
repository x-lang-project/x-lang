mod exceptions;

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
    fn compile_goto_terminator(
        &self,
        target_block_identifier: &BlockIdentifier,
    ) -> Result<(), CodeGenerationError> {
        let next_basic_block = self
            .llvm_blocks
            .get(&target_block_identifier.0)
            .copied()
            .ok_or(CodeGenerationError::MissingLlvmBlock(
                target_block_identifier.0,
            ))?;
        self.br(next_basic_block)?;

        Ok(())
    }

    fn compile_return_terminator(&self) -> Result<(), CodeGenerationError> {
        if self.body.is_memoised {
            self.emit_memo_store()?;
        }

        let mir_return_type = self
            .body
            .locals
            .get(&self.body.return_local)
            .ok_or_else(|| {
                CodeGenerationError::LlvmInternalError(
                    backend_messages::missing_return_slot_layout_tracking_record(),
                )
            })?
            .value_type
            .clone();

        if self.body.is_throws {
            let payload = if mir_return_type == Type::Void {
                self.codegen.resolve_llvm_type(&Type::Void)?.const_zero()
            } else if let Some(return_pointer) = self.frame.pointer(self.body.return_local) {
                let element_type = self.codegen.resolve_llvm_type(&mir_return_type)?;

                self.load(element_type, return_pointer, "throws_success_value")?
            } else {
                self.codegen
                    .resolve_llvm_type(&mir_return_type)?
                    .const_zero()
            };

            let result =
                self.throws_result(true, payload, self.pointer_type().const_null().into())?;
            self.return_value(Some(&result))?;
        } else if mir_return_type == Type::Void {
            if self.function_name == x_ast::ENTRY_POINT_NAME {
                let current = self.current_function()?;

                if current.get_type().get_return_type().is_some() {
                    let status = self.i32().const_zero();
                    self.return_value(Some(&status))?;
                } else {
                    self.return_value(None)?;
                }
            } else {
                self.return_value(None)?;
            }
        } else if let Some(return_pointer) = self.frame.pointer(self.body.return_local) {
            let element_type = self.codegen.resolve_llvm_type(&mir_return_type)?;
            let return_value = self.load(element_type, return_pointer, "ret_val")?;
            self.return_value(Some(&return_value))?;
        } else {
            self.return_value(None)?;
        }

        Ok(())
    }

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
