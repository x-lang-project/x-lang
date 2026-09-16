mod access;
mod descriptors;
mod mutation;

use inkwell::IntPredicate;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue};

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_ast::intrinsics::{BuiltinCall, BuiltinKind, SliceOperation};
use x_diagnostics::backend_messages;
use x_mir::Operand;

mod i32;
mod scalar;

#[derive(Clone)]
struct SliceValue<'ctx> {
    data: PointerValue<'ctx>,
    len: IntValue<'ctx>,
    element: BasicTypeEnum<'ctx>,
    element_ast: Type,
}

struct CopyEndpoint<'ctx> {
    pointer: PointerValue<'ctx>,
    index: IntValue<'ctx>,
}

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn emit_from_raw_parts(
        &self,
        function: &str,
        arguments: &[Operand],
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let [data, length] = self.arguments(function, arguments).exact()?;
        let data = self.load_operand_value(data.operand())?;
        let length = length.value(&Type::USize)?;

        self.emit_slice_descriptor(data, length)
    }

    pub(super) fn emit_resolved_slice_method(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let BuiltinKind::Slice(operation) = call.kind else {
            return Err(CodeGenerationError::MissingCalleeSymbol(call.name.clone()));
        };

        self.emit_slice_operation(&call.name, arguments, operation)
    }

    fn emit_slice_operation(
        &self,
        function: &str,
        arguments: &[Operand],
        method: SliceOperation,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        if matches!(
            method,
            SliceOperation::AsConst
                | SliceOperation::Iter
                | SliceOperation::Next
                | SliceOperation::Remaining
                | SliceOperation::Sub
                | SliceOperation::SplitAt
        ) {
            return Err(CodeGenerationError::InvalidMir(
                backend_messages::slice_view_requires_explicit_identity_lifetime_and_mutability(
                    function,
                ),
            ));
        }

        if matches!(
            method,
            SliceOperation::All | SliceOperation::Any | SliceOperation::Reduce
        ) {
            return Err(CodeGenerationError::LlvmInternalError(
                backend_messages::slice_callback_lowering_unsupported(function),
            ));
        }

        if matches!(
            method,
            SliceOperation::HasDuplicateBy | SliceOperation::UniqueBy
        ) {
            self.arguments(function, arguments).exact::<1>()?;
        }

        let slice = self.slice_receiver(function, arguments)?;

        match method {
            SliceOperation::Len => Ok(slice.len.into()),
            SliceOperation::IsEmpty => self.emit_slice_is_empty(slice),
            SliceOperation::AsPointer | SliceOperation::AsMutPointer => Ok(slice.data.into()),
            SliceOperation::First => {
                let has_items = self.icmp(
                    IntPredicate::NE,
                    slice.len,
                    slice.len.get_type().const_zero(),
                    "slice_first_has_items",
                )?;

                self.option_pointer(has_items, slice.data, "slice_first_option")
            }
            SliceOperation::AsConst
            | SliceOperation::Iter
            | SliceOperation::Next
            | SliceOperation::Remaining
            | SliceOperation::Sub
            | SliceOperation::SplitAt => Err(CodeGenerationError::InvalidMir(
                backend_messages::slice_view_requires_explicit_identity_lifetime_and_mutability(
                    function,
                ),
            )),
            SliceOperation::Last => self.emit_slice_last(slice),
            SliceOperation::Sum => self.emit_slice_sum(slice, arguments.get(1)),
            SliceOperation::All | SliceOperation::Any | SliceOperation::Reduce => {
                Err(CodeGenerationError::LlvmInternalError(
                    backend_messages::slice_callback_lowering_unsupported(function),
                ))
            }
            SliceOperation::Contains => self.emit_slice_contains(function, arguments, slice),
            SliceOperation::HasDuplicateBy => self
                .emit_typed_slice_has_duplicate(
                    slice.data,
                    slice.len,
                    slice.element,
                    &slice.element_ast,
                )
                .map(|value| value.into()),
            SliceOperation::UniqueBy => self
                .emit_typed_slice_has_duplicate(
                    slice.data,
                    slice.len,
                    slice.element,
                    &slice.element_ast,
                )
                .and_then(|value| self.integer_not(value, "slice_unique_by"))
                .map(|value| value.into()),
            SliceOperation::Fill => self.emit_slice_fill(function, arguments, slice),
            SliceOperation::Swap => self.emit_slice_swap(function, arguments, slice),
            SliceOperation::CopyWithin => self.emit_slice_copy_within(function, arguments, slice),
            SliceOperation::CopyFrom => self.emit_slice_copy_from(function, arguments, slice),
            SliceOperation::Get | SliceOperation::GetMut => {
                self.emit_slice_get(function, arguments, slice)
            }
        }
    }

    fn emit_slice_contains(
        &self,
        function: &str,
        arguments: &[Operand],
        slice: SliceValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let [_, needle] = self.arguments(function, arguments).exact()?;
        let needle_ast = self.operand_ast_type(needle.operand());
        let needle = if matches!(needle_ast, Some(Type::Reference { .. })) {
            let pointer = self.pointer_argument(needle.operand(), "slice_contains_needle_ref")?;

            self.load(slice.element, pointer, "slice_contains_needle")?
        } else {
            self.load_operand_value_with_type(needle.operand(), &slice.element_ast)?
        };

        self.emit_typed_slice_contains(&slice, needle)
            .map(Into::into)
    }
}
