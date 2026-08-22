use super::*;

impl TypeChecker {
    pub(super) fn type_invalidates_pure_memory_contract(&mut self, value_type: &Type) -> bool {
        self.pure_memory_type_walk(value_type, &mut std::collections::HashSet::new(), &mut 4096)
    }

    fn pure_memory_type_walk(
        &mut self,
        value_type: &Type,
        active: &mut std::collections::HashSet<Type>,
        budget: &mut usize,
    ) -> bool {
        if *budget == 0 || active.len() >= 64 {
            return true;
        }

        *budget -= 1;
        let resolved = self
            .resolve_type(value_type)
            .unwrap_or_else(|_| self.resolve(value_type));

        if !active.insert(resolved.clone()) {
            return false;
        }

        let restricted = match &resolved {
            Type::RawPointer { .. } => true,
            Type::Reference {
                is_mut,
                is_unique,
                inner,
                ..
            } => *is_mut || *is_unique || self.pure_memory_type_walk(inner, active, budget),
            Type::Array(inner)
            | Type::FixedSizedArray(inner, _)
            | Type::Simd { element: inner, .. } => {
                self.pure_memory_type_walk(inner, active, budget)
            }
            Type::Tuple(items) => items
                .iter()
                .any(|value_type| self.pure_memory_type_walk(value_type, active, budget)),
            Type::Custom(name) => {
                let fields = self
                    .resolve_struct_name(name)
                    .ok()
                    .and_then(|key| self.symbols.records.fields().get(&key).cloned());

                fields.is_some_and(|fields| {
                    fields.iter().any(|(_, value_type)| {
                        self.pure_memory_type_walk(value_type, active, budget)
                    })
                })
            }
            Type::GenericInstance {
                name,
                type_arguments,
            } => {
                if type_arguments
                    .iter()
                    .any(|value_type| self.pure_memory_type_walk(value_type, active, budget))
                {
                    true
                } else {
                    let definition = self
                        .resolve_generic_struct_key(name)
                        .and_then(|key| self.symbols.records.templates().get(&key).cloned());

                    if let Some(definition) = definition {
                        let substitutions = definition
                            .generic_parameters
                            .as_ref()
                            .into_iter()
                            .flatten()
                            .filter_map(x_ast::GenericParameter::non_lifetime_name)
                            .cloned()
                            .zip(type_arguments.iter().cloned())
                            .collect::<HashMap<_, _>>();

                        definition.fields.iter().any(|(_, value_type)| {
                            let concrete = self.substitute_type(value_type, &substitutions);

                            self.pure_memory_type_walk(&concrete, active, budget)
                        })
                    } else {
                        false
                    }
                }
            }
            _ => false,
        };

        active.remove(&resolved);

        restricted
    }
}

pub(super) fn type_contains_runtime_handle(value_type: &Type) -> bool {
    type_contains_restricted_memory(value_type, false)
}

pub(super) fn type_contains_restricted_memory(value_type: &Type, allow_shared_refs: bool) -> bool {
    match value_type {
        Type::RawPointer { .. } => true,
        Type::Reference {
            is_mut, is_unique, ..
        } => !allow_shared_refs || *is_mut || *is_unique,
        Type::Array(inner) | Type::FixedSizedArray(inner, _) => {
            type_contains_restricted_memory(inner, allow_shared_refs)
        }
        Type::Tuple(items)
        | Type::GenericInstance {
            type_arguments: items,
            ..
        } => items
            .iter()
            .any(|item| type_contains_restricted_memory(item, allow_shared_refs)),
        _ => false,
    }
}

pub(super) fn validate_external_c_abi(name: &str, abi: Option<&str>) -> Result<(), TypeError> {
    let label = abi.map(|label| label.trim().trim_matches(['"', '\'']));

    if label.is_some_and(|label| label.eq_ignore_ascii_case("c")) {
        Ok(())
    } else {
        Err(TypeError::inference(
            FrontendMessage::UnsupportedExternAbi {
                name: name.to_string(),
                found: label.unwrap_or("no ABI").to_string(),
            },
        ))
    }
}

pub(super) fn external_ffi_type_is_supported(value_type: &Type, allow_void: bool) -> bool {
    match value_type {
        Type::Void => allow_void,
        Type::Boolean
        | Type::Character
        | Type::I8
        | Type::U8
        | Type::I16
        | Type::U16
        | Type::I32
        | Type::U32
        | Type::I64
        | Type::U64
        | Type::ISize
        | Type::USize
        | Type::F32
        | Type::F64 => true,
        Type::RawPointer { inner, .. } => external_ffi_type_is_supported(inner, true),
        _ => false,
    }
}
