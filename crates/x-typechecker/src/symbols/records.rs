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

pub(crate) struct RecordReservation {
    name: String,
}

impl RecordCatalogue {
    pub(crate) fn fields(&self) -> &HashMap<String, Vec<(String, Type)>> {
        &self.fields
    }

    pub(crate) fn templates(&self) -> &HashMap<String, StructDefinition> {
        &self.templates
    }

    pub(crate) fn replace_fields(&mut self, name: &str, fields: Vec<(String, Type)>) -> Option<()> {
        *self.fields.get_mut(name)? = fields;

        Some(())
    }

    pub(crate) fn layout(&self, name: &str) -> Option<&Layout> {
        self.layouts.get(name)
    }

    pub(crate) fn instantiations(&self) -> &HashMap<(String, Vec<Type>), String> {
        &self.instantiations
    }

    pub(crate) fn arguments(&self, name: &str) -> Option<&Vec<Type>> {
        self.arguments.get(name)
    }

    pub(crate) fn declare_template(&mut self, name: String, definition: StructDefinition) {
        self.layouts.insert(name.clone(), definition.layout);
        self.templates.insert(name, definition);
    }

    pub(crate) fn index_declaration(&mut self, definition: StructDefinition) {
        self.layouts
            .insert(definition.name.clone(), definition.layout);

        if definition.generic_parameters.is_some() {
            self.templates
                .entry(definition.name.clone())
                .and_modify(|existing| merge_fields(&mut existing.fields, &definition.fields))
                .or_insert(definition);
        } else {
            self.fields
                .entry(definition.name)
                .and_modify(|existing| merge_fields(existing, &definition.fields))
                .or_insert(definition.fields);
        }
    }

    // Reserve the identity before recursive field substitution.
    pub(crate) fn reserve_instantiation(
        &mut self,
        value_type: (String, Vec<Type>),
        name: String,
        layout: Layout,
    ) -> RecordReservation {
        self.arguments.insert(name.clone(), value_type.1.clone());
        self.instantiations.insert(value_type, name.clone());
        self.layouts.insert(name.clone(), layout);
        self.fields.insert(name.clone(), Vec::new());

        RecordReservation { name }
    }

    pub(crate) fn complete_instantiation(
        &mut self,
        reservation: RecordReservation,
        fields: Vec<(String, Type)>,
    ) {
        self.fields.insert(reservation.name, fields);
    }
}

fn merge_fields(existing: &mut Vec<(String, Type)>, incoming: &[(String, Type)]) {
    for (name, value_type) in incoming {
        if !existing
            .iter()
            .any(|(existing_name, _)| existing_name == name)
        {
            existing.push((name.clone(), value_type.clone()));
        }
    }
}
