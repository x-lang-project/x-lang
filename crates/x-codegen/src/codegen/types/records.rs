use inkwell::types::StructType;

use super::super::CodeGeneration;
use x_ast::{Layout, Type};

impl<'ctx> CodeGeneration<'ctx> {
    pub(in crate::codegen) fn type_path_ends_with(candidate: &str, requested: &str) -> bool {
        let candidate_segments = candidate.split("::");
        let requested_segments = requested.split("::");
        let Some(skip) = candidate_segments
            .clone()
            .count()
            .checked_sub(requested_segments.clone().count())
        else {
            return false;
        };

        candidate_segments.skip(skip).eq(requested_segments)
    }
}
