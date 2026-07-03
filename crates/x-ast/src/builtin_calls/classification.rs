use crate::Type;
use crate::builtin_calls::{
    AlignOperation, BuiltinKind, CoreMemoryOperation, HeapAllocation, NumberOperation,
    PointerConstructor, PointerOperation, SliceOperation, is_slice_operation,
};
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

    fn runtime_operation(&self) -> Option<BuiltinKind> {
        let control = self
            .scalar_operation
            .and_then(|name| Intrinsic::resolve(name, IntrinsicScope::Source))
            .filter(|intrinsic| matches!(intrinsic, Intrinsic::Control(_)));

        if let Some(control) = control {
            Some(BuiltinKind::Intrinsic(control))
        } else if is_abort_call(self.base) {
            Some(BuiltinKind::Intrinsic(Intrinsic::Control(
                ControlIntrinsic::Abort,
            )))
        } else if self.base == "as_str" && self.receiver == Some(&Type::String) {
            Some(BuiltinKind::StringView)
        } else if is_builtin_print_call(self.base, "print") {
            Some(BuiltinKind::Print)
        } else if is_builtin_print_call(self.base, "println") {
            Some(BuiltinKind::Println)
        } else {
            None
        }
    }

    fn numeric_intrinsic(&self) -> Option<BuiltinKind> {
        let operation = self.scalar_operation?;
        let intrinsic = Intrinsic::resolve(operation, IntrinsicScope::Source)?;

        match intrinsic {
            Intrinsic::Float(_)
            | Intrinsic::Overflow(_)
            | Intrinsic::Rotate(_)
            | Intrinsic::Bit(_) => Some(BuiltinKind::Intrinsic(intrinsic)),
            Intrinsic::Control(_) | Intrinsic::Memory(_) | Intrinsic::Layout(_) => None,
        }
    }

    fn collection_operation(&self) -> Option<BuiltinKind> {
        match self.base {
            "slice::from_raw_parts" | "slice::from_raw_parts_mut" => {
                Some(BuiltinKind::FromRawParts)
            }
            "align::is_power_of_two"
            | "align::is_aligned"
            | "align::align_up"
            | "align::page_align_up"
            | "align::align_ptr"
            | "align::align_down"
            | "align::page_align_down"
            | "align::padding_needed"
            | "align::max_align"
            | "align::min_align" => AlignOperation::resolve(self.leaf).map(BuiltinKind::Align),
            "num::saturating_sub_usize"
            | "num::checked_mul_usize"
            | "num::checked_add_usize"
            | "num::checked_sub_usize"
            | "num::abs_i32"
            | "num::min"
            | "num::max"
            | "num::clamp" => NumberOperation::resolve(self.leaf).map(BuiltinKind::Number),
            _ if self.base == self.leaf
                && self.slice_receiver()
                && is_slice_operation(self.leaf) =>
            {
                SliceOperation::resolve(self.leaf).map(BuiltinKind::Slice)
            }
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
