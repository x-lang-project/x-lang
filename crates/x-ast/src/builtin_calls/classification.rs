use crate::Type;
use crate::builtin_calls::AlignOperation;
use crate::builtin_calls::BuiltinKind;
use crate::builtin_calls::CoreMemoryOperation;
use crate::builtin_calls::HeapAllocation;
use crate::builtin_calls::NumberOperation;
use crate::builtin_calls::PointerConstructor;
use crate::builtin_calls::PointerOperation;
use crate::builtin_calls::SliceOperation;
use crate::intrinsics::{ControlIntrinsic, Intrinsic, IntrinsicScope, LayoutIntrinsic};
use crate::intrinsics::{
    function_leaf_name, is_abort_call, is_builtin_print_call, strip_explicit_type_arguments,
};
use receivers::{array_receiver, string_receiver};

mod receivers;
pub(super) struct CallContext<'a> {
    pub base: &'a str,
    pub leaf: &'a str,
    pub pointer_context: bool,
    scalar_operation: Option<&'a str>,
    receiver: Option<&'a Type>,
}

impl<'a> CallContext<'a> {
    pub fn new(name: &'a str, arguments: &'a [Type]) -> Self {
        let base = strip_explicit_type_arguments(name);
        let leaf = function_leaf_name(name);
        let receiver = arguments.first();
        let pointer_context = base
            .strip_prefix("ptr::")
            .is_some_and(|operator| !operator.contains("::"))
            || (base == leaf
                && receiver
                    .is_some_and(|value_type| matches!(value_type, Type::RawPointer { .. })));
        let operation = base.strip_prefix("intrinsic::").unwrap_or(base);

        Self {
            base,
            leaf,
            pointer_context,
            scalar_operation: (!operation.contains("::")).then_some(operation),
            receiver,
        }
    }

    fn collection_operation(&self) -> Option<BuiltinKind> {
        match self.base {
            _ => None,
        }
    }

    fn slice_receiver(&self) -> bool {
        self.receiver.is_some_and(array_receiver)
            || (self.base == "len" && self.receiver.is_some_and(string_receiver))
    }

    fn memory_operation(&self) -> Option<BuiltinKind> {
        if self.scalar_operation == Some("fill_i32") {
            return Some(BuiltinKind::FillI32);
        }

        match self.base {
            "mem::copy_nonoverlapping"
            | "mem::copy_nonoverlapping_bytes"
            | "mem::copy"
            | "mem::move_objects"
            | "mem::move_bytes"
            | "mem::set_bytes"
            | "mem::zero_objects"
            | "mem::zero_bytes"
            | "mem::fill"
            | "mem::checked_array_size" => {
                CoreMemoryOperation::resolve(self.leaf).map(BuiltinKind::CoreMemory)
            }
            "heap::alloc" | "heap::alloc_zeroed"
                if matches!(self.leaf, "alloc" | "alloc_zeroed") =>
            {
                Some(BuiltinKind::HeapAllocate(if self.leaf == "alloc_zeroed" {
                    HeapAllocation::Zeroed
                } else {
                    HeapAllocation::Uninitialized
                }))
            }
            "size_of" | "mem::size_of" | "intrinsic::size_of" => Some(BuiltinKind::Intrinsic(
                Intrinsic::Layout(LayoutIntrinsic::SizeOf),
            )),
            "align_of" | "mem::align_of" | "intrinsic::align_of" => Some(BuiltinKind::Intrinsic(
                Intrinsic::Layout(LayoutIntrinsic::AlignOf),
            )),
            "offset_of" | "mem::offset_of" | "intrinsic::offset_of" => Some(
                BuiltinKind::Intrinsic(Intrinsic::Layout(LayoutIntrinsic::OffsetOf)),
            ),
            "mem::swap" if self.leaf == "swap" => Some(BuiltinKind::MemSwap),
            "mem::replace" if self.leaf == "replace" => Some(BuiltinKind::MemReplace),
            "ptr::assert_non_null" if self.leaf == "assert_non_null" => {
                Some(BuiltinKind::AssertNonNull)
            }
            "ptr::assert_aligned" if self.leaf == "assert_aligned" => {
                Some(BuiltinKind::AssertAligned)
            }
            _ if self.pointer_context
                && matches!(self.leaf, "null" | "null_mut" | "dangling" | "dangling_mut") =>
            {
                PointerConstructor::resolve(self.leaf).map(BuiltinKind::PointerNamespace)
            }
            _ if matches!(self.scalar_operation, Some("memcpy" | "memmove" | "memset")) => {
                Intrinsic::resolve(self.scalar_operation?, IntrinsicScope::Source)
                    .map(BuiltinKind::Intrinsic)
            }
            _ => None,
        }
    }
}
