use super::*;

pub(super) fn storage_is_raw_pointer_free(
    value_type: &Type,
    layouts: &crate::codegen::records::RecordCatalogue,
    active: &mut std::collections::HashSet<String>,
    depth: usize,
) -> bool {
    if depth >= 128 {
        return false;
    }

    match value_type {
        Type::RawPointer { .. } | Type::Unknown | Type::Infer(_) | Type::TypeParameter(_) => false,
        Type::Reference { inner: base, .. }
        | Type::FixedSizedArray(base, _)
        | Type::Simd { element: base, .. } => {
            storage_is_raw_pointer_free(base, layouts, active, depth + 1)
        }
        Type::Tuple(items) => items
            .iter()
            .all(|value_type| storage_is_raw_pointer_free(value_type, layouts, active, depth + 1)),
        Type::Array(base) => storage_is_raw_pointer_free(base, layouts, active, depth + 1),
        Type::Function {
            parameters,
            return_value,
        } => {
            parameters.iter().all(|value_type| {
                storage_is_raw_pointer_free(value_type, layouts, active, depth + 1)
            }) && storage_is_raw_pointer_free(return_value, layouts, active, depth + 1)
        }
        Type::Custom(name) | Type::GenericInstance { name, .. } => {
            let Some(record) = layouts.get(name) else {
                return false;
            };

            if !active.insert(name.clone()) {
                return true;
            }

            let clean = record.fields().iter().all(|(_, value_type)| {
                storage_is_raw_pointer_free(value_type, layouts, active, depth + 1)
            }) && match value_type {
                Type::GenericInstance { type_arguments, .. } => {
                    type_arguments.iter().all(|argument| {
                        storage_is_raw_pointer_free(argument, layouts, active, depth + 1)
                    })
                }
                _ => true,
            };
            active.remove(name);

            clean
        }
        _ => true,
    }
}

pub(super) fn unique_noalias_is_proved(
    parameters: &[Type],
    layouts: &crate::codegen::records::RecordCatalogue,
) -> bool {
    parameters.iter().all(|value_type| {
        storage_is_raw_pointer_free(
            value_type,
            layouts,
            &mut std::collections::HashSet::new(),
            0,
        )
    })
}

pub(super) fn pure_no_memory_parameter(value_type: &Type) -> bool {
    match value_type {
        Type::Reference { .. }
        | Type::RawPointer { .. }
        | Type::Array(_)
        | Type::String
        | Type::Function { .. } => false,
        Type::FixedSizedArray(inner, _) | Type::Simd { element: inner, .. } => {
            pure_no_memory_parameter(inner)
        }
        Type::Tuple(items) => items.iter().all(pure_no_memory_parameter),
        Type::GenericInstance { .. } | Type::Custom(_) | Type::Unknown | Type::Infer(_) => false,
        _ => true,
    }
}
