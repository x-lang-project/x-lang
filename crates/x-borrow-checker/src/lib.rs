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

mod control_flow;

mod reference_value;
mod return_summaries;

pub use reference_value::ReferenceProjection;

// Origin collection and lifetime checks must share this depth limit.
