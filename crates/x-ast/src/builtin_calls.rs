use crate::Type;
use crate::intrinsics::parse_generic_type_argument;

mod scalar_operations;
mod slice_operations;

pub use scalar_operations::{AlignOperation, NumberOperation};
pub use slice_operations::{SliceOperation, SliceReceiverAccess};
