use std::collections::HashMap;

use x_ast::{Layout, StructDefinition, Type};

#[derive(Debug, Default)]
pub(crate) struct RecordCatalogue {
    fields: HashMap<String, Vec<(String, Type)>>,
    templates: HashMap<String, StructDefinition>,
    layouts: HashMap<String, Layout>,
    instantiations: HashMap<(String, Vec<Type>), String>,
    arguments: HashMap<String, Vec<Type>>,
}

impl RecordCatalogue {
    pub(crate) fn fields(&self) -> &HashMap<String, Vec<(String, Type)>> {
        &self.fields
    }

    pub(crate) fn templates(&self) -> &HashMap<String, StructDefinition> {
        &self.templates
    }

    pub(crate) fn instantiations(&self) -> &HashMap<(String, Vec<Type>), String> {
        &self.instantiations
    }

    pub(crate) fn arguments(&self, name: &str) -> Option<&Vec<Type>> {
        self.arguments.get(name)
    }
}
