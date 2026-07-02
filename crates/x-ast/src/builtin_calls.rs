use crate::Type;
use crate::intrinsics::parse_generic_type_argument;

mod memory_operations;
mod pointer_operations;
pub use memory_operations::CoreMemoryOperation;
mod scalar_operations;
mod slice_operations;
pub use pointer_operations::{HeapAllocation, PointerConstructor, PointerOperation};
pub use scalar_operations::{AlignOperation, NumberOperation};
pub use slice_operations::{SliceOperation, SliceReceiverAccess};
