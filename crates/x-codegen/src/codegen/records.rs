use std::collections::HashMap;

use x_ast::{Layout, StructDefinition, StructInvariant, Type};

#[derive(Default)]
pub(crate) struct RecordCatalogue {
    declarations: HashMap<String, RecordMetadata>,
}

pub(crate) struct RecordMetadata {
    fields: Vec<(String, Type)>,
    indices: HashMap<String, u32>,
    layout: Layout,
    invariant: Option<StructInvariant>,
}

impl RecordCatalogue {
    pub(crate) fn resolve_name(&self, requested: &str) -> Option<&str> {
        if let Some((name, _)) = self.declarations.get_key_value(requested) {
            return Some(name);
        }

        if requested.contains("::") {
            return None;
        }

        let mut matches = self
            .declarations
            .keys()
            .filter(|candidate| super::CodeGeneration::type_path_ends_with(candidate, requested));
        let first = matches.next()?;

        matches.next().is_none().then_some(first.as_str())
    }
}

impl RecordMetadata {
    pub(crate) fn layout(&self) -> Layout {
        self.layout
    }

    pub(crate) fn invariant(&self) -> Option<&StructInvariant> {
        self.invariant.as_ref()
    }

    pub(crate) fn fields(&self) -> &[(String, Type)] {
        &self.fields
    }

    pub(crate) fn field_index(&self, name: &str) -> Option<u32> {
        self.indices.get(name).copied()
    }
}
