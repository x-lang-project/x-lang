mod model;
mod names;

pub use model::ControlIntrinsic;
pub use model::IntrinsicScope;
pub use model::IntrinsicSignature;
pub use model::IntrinsicVisibility;
pub use model::LayoutIntrinsic;
pub use model::MemoryIntrinsic;

pub use names::function_leaf_name;
pub use names::function_owner_name;
pub use names::is_abort_call;
pub use names::is_builtin_print_call;
pub use names::is_catch_call;
pub use names::is_heap_allocate_call;
pub use names::is_mem_namespace_call;
pub use names::is_pointer_namespace_named_call;
pub use names::parse_generic_type_argument;
pub use names::strip_explicit_type_arguments;
