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

    pub fn resolve_struct_field_type(&self, struct_name: &str, field_name: &str) -> Option<Type> {
        let canonical_name = self.resolve_struct_name(struct_name).unwrap_or(struct_name);

        self.records
            .get(canonical_name)
            .and_then(|record| record.field_type(field_name))
            .cloned()
    }

    pub(crate) fn soa_fields(&self, value_type: &Type) -> Option<Vec<(String, Type)>> {
        let name = match value_type {
            Type::Custom(name) | Type::GenericInstance { name, .. } => name,
            _ => return None,
        };

        let canonical = self.resolve_struct_name(name).unwrap_or(name);

        let record = self.records.get(canonical)?;

        (record.layout() == Layout::SoA).then(|| record.fields().to_vec())
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
