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
    fn memory_operation(&self) -> Option<BuiltinKind> {
        if self.scalar_operation == Some("fill_i32") {
            return Some(BuiltinKind::FillI32);
        }

        match self.base {
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
