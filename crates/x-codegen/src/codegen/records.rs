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
