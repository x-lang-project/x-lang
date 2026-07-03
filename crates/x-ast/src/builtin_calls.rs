use crate::Type;
use crate::intrinsics::parse_generic_type_argument;

mod classification;
mod memory_operations;
mod pointer_operations;
pub use memory_operations::CoreMemoryOperation;
mod scalar_operations;
mod slice_operations;
pub use pointer_operations::{HeapAllocation, PointerConstructor, PointerOperation};
pub use scalar_operations::{AlignOperation, NumberOperation};
pub use slice_operations::{SliceOperation, SliceReceiverAccess};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinKind {
    Pointer(PointerOperation),
    StringView,
    StringBytes,
    Print,
    Println,
    Intrinsic(crate::intrinsics::Intrinsic),
    FromRawParts,
    Align(AlignOperation),
    Number(NumberOperation),
    Slice(SliceOperation),
    FillI32,
    CoreMemory(CoreMemoryOperation),
    HeapAllocate(HeapAllocation),
    MemSwap,
    MemReplace,
    AssertNonNull,
    AssertAligned,
    PointerNamespace(PointerConstructor),
}

pub fn is_slice_operation(leaf: &str) -> bool {
    SliceOperation::resolve(leaf).is_some()
}
