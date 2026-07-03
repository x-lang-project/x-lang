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
