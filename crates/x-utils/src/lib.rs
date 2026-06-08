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

pub mod artifacts;
pub mod optimization;
pub mod project;
pub mod target;
pub mod temporary;
pub mod tools;

const FNV1A_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A_PRIME: u64 = 0x100000001b3;
