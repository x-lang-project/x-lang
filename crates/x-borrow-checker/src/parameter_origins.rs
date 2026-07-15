use std::collections::HashMap;

use super::{BorrowCheckError, ReferenceTarget, ReferenceValue, Region};
use crate::ReferenceProjection;
use x_ast::{StructDefinition, Type};
use x_diagnostics::borrow_messages;

pub(super) fn parameter(
    name: &str,
    value_type: &Type,
    layouts: &HashMap<String, StructDefinition>,
    origins: &mut HashMap<String, Type>,
) -> Result<ReferenceValue, BorrowCheckError> {
    ParameterOrigins { layouts, origins }.collect(name, value_type, 0)
}

struct ParameterOrigins<'a> {
    layouts: &'a HashMap<String, StructDefinition>,
    origins: &'a mut HashMap<String, Type>,
}

impl ParameterOrigins<'_> {
    fn collect(
        &mut self,
        name: &str,
        value_type: &Type,
        depth: usize,
    ) -> Result<ReferenceValue, BorrowCheckError> {
        if depth >= super::MAX_AGGREGATE_LAYOUT_DEPTH {
            return Err(BorrowCheckError::Invariant(
                borrow_messages::BorrowInvariant::ParameterAggregateLayoutDepthExceeded {
                    maximum: super::MAX_AGGREGATE_LAYOUT_DEPTH,
                },
            ));
        }

        let mut value = ReferenceValue::default();

        match value_type {
            Type::Reference { is_mut, .. } | Type::RawPointer { is_mut, .. } => {
                self.origins.insert(name.to_string(), value_type.clone());
                value.targets.push(ReferenceTarget {
                    binding_generation: 0,
                    projection: Vec::new(),
                    target_variable: name.to_string(),
                    origin_region: Region(0),
                    is_mut: *is_mut,
                });
                value.origin_known = true;
            }
            Type::Tuple(items) => {
                for (index, value_type) in items.iter().enumerate() {
                    value.fields.insert(
                        ReferenceProjection::Index(index as i64),
                        self.collect(&format!("{name}.{index}"), value_type, depth + 1)?,
                    );
                }
            }
            Type::FixedSizedArray(_, 0) => {}
            Type::Array(inner) | Type::FixedSizedArray(inner, _) => {
                value.fields.insert(
                    ReferenceProjection::AnyIndex,
                    self.collect(&format!("{name}.*"), inner, depth + 1)?,
                );
            }
            Type::Custom(_) | Type::GenericInstance { .. } => {
                if let Some(fields) = fields(value_type, self.layouts) {
                    for (field, value_type) in fields {
                        value.fields.insert(
                            ReferenceProjection::Field(field.clone()),
                            self.collect(&format!("{name}.{field}"), &value_type, depth + 1)?,
                        );
                    }
                }
            }
            _ => {}
        }

        Ok(value)
    }
}

pub(super) fn fields(
    value_type: &Type,
    layouts: &HashMap<String, StructDefinition>,
) -> Option<Vec<(String, Type)>> {
    let name = match value_type {
        Type::Custom(name) | Type::GenericInstance { name, .. } => name,
        _ => return None,
    };

    let layout = layouts.get(name)?;
    let substitutions = match value_type {
        Type::GenericInstance { type_arguments, .. } => layout
            .generic_parameters
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter_map(x_ast::GenericParameter::non_lifetime_name)
            .cloned()
            .zip(type_arguments.iter().cloned())
            .collect(),
        _ => HashMap::new(),
    };

    Some(
        layout
            .fields
            .iter()
            .map(|(name, value_type)| (name.clone(), substitute(value_type, &substitutions)))
            .collect(),
    )
}

fn substitute(value_type: &Type, substitutions: &HashMap<String, Type>) -> Type {
    let mut value_type = value_type.clone();

    match &mut value_type {
        Type::TypeParameter(name) | Type::Custom(name) if substitutions.contains_key(name) => {
            if let Some(value) = substitutions.get(name) {
                return value.clone();
            }
        }
        Type::Reference { inner, .. }
        | Type::RawPointer { inner, .. }
        | Type::Array(inner)
        | Type::FixedSizedArray(inner, _)
        | Type::Simd { element: inner, .. } => **inner = substitute(inner, substitutions),
        Type::Tuple(items)
        | Type::GenericInstance {
            type_arguments: items,
            ..
        } => {
            for item in items {
                *item = substitute(item, substitutions);
            }
        }
        Type::Function {
            parameters,
            return_value,
        } => {
            for parameter in parameters {
                *parameter = substitute(parameter, substitutions);
            }
            **return_value = substitute(return_value, substitutions);
        }
        _ => {}
    }

    value_type
}
