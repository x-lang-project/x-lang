#![cfg_attr(
    not(test),
    deny(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented,
        clippy::unreachable,
        clippy::unwrap_used
    )
)]

use std::sync::Arc;

use context::FunctionLifetimeContext;

use program::nominal_owner_name;

use type_properties::{declared_reference_lifetime, has_non_lifetime_generic_parameter};

mod context;
mod control_flow;

mod ownership;
mod parameter_origins;

mod program;

mod reference_returns;
mod reference_value;
mod return_summaries;

mod storage;
mod type_properties;

pub use reference_value::{ReferenceProjection, ReferenceValue};

// Origin collection and lifetime checks must share this depth limit.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Region(pub usize);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReferenceTarget {
    pub binding_generation: usize,
    pub projection: Vec<ReferenceProjection>,
    pub target_variable: String,
    pub origin_region: Region,
    pub is_mut: bool,
}
