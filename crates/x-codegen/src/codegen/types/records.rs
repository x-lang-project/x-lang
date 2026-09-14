use inkwell::types::StructType;

use super::super::CodeGeneration;
use x_ast::{Layout, Type};

impl<'ctx> CodeGeneration<'ctx> {
    pub fn resolve_struct_type(&self, name: &str) -> Option<StructType<'ctx>> {
        if let Some(value_type) = self.module.get_struct_type(name) {
            return Some(value_type);
        }

        self.resolve_struct_name(name)
            .and_then(|canonical| self.module.get_struct_type(canonical))
    }

    pub fn resolve_struct_name(&self, name: &str) -> Option<&str> {
        self.records.resolve_name(name)
    }

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
