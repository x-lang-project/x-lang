mod types;

use crate::ReferenceProjection;
use crate::ReferenceTarget;
use crate::ReferenceValue;
use crate::Region;
use types::stored_children;
use x_ast::{Expression, Type};

struct UniqueReborrow {
    referents: Vec<ReferenceTarget>,
    sources: Vec<ReferenceTarget>,
}

fn same_storage(left: &ReferenceTarget, right: &ReferenceTarget) -> bool {
    left.origin_region == right.origin_region
        && left.binding_generation == right.binding_generation
        && left.target_variable == right.target_variable
}

fn projections_overlap(left: &[ReferenceProjection], right: &[ReferenceProjection]) -> bool {
    left.iter().zip(right).all(|(left, right)| {
        left == right
            || matches!(left, ReferenceProjection::AnyIndex)
            || matches!(right, ReferenceProjection::AnyIndex)
    })
}
