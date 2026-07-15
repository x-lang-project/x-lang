use super::{BorrowChecker, MAX_AGGREGATE_LAYOUT_DEPTH, parameter_origins};
use x_ast::{GenericParameter, Lifetime, Type};

impl BorrowChecker {
    pub(super) fn is_copy_type(&self, value_type: &Type) -> bool {
        self.is_copy_type_inner(value_type, 0)
    }

    fn is_copy_type_inner(&self, current_value_type: &Type, depth: usize) -> bool {
        if depth >= MAX_AGGREGATE_LAYOUT_DEPTH {
            return false;
        }

        match current_value_type {
            Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::ISize
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::USize
            | Type::F32
            | Type::F64
            | Type::Boolean
            | Type::Character
            | Type::String
            | Type::Unknown
            | Type::Infer(_)
            | Type::RawPointer { .. } => true,
            Type::Reference { is_unique, .. } => !is_unique,
            Type::Tuple(items) => items
                .iter()
                .all(|value_type| self.is_copy_type_inner(value_type, depth + 1)),
            Type::FixedSizedArray(inner, _) => self.is_copy_type_inner(inner, depth + 1),
            Type::Custom(name) | Type::GenericInstance { name, .. } => {
                self.context
                    .struct_layouts
                    .get(name)
                    .is_some_and(|definition| {
                        definition.derives.iter().any(|derive| derive == "Copy")
                    })
                    && parameter_origins::fields(current_value_type, &self.context.struct_layouts)
                        .is_some_and(|fields| {
                            fields.iter().all(|(_, value_type)| {
                                self.is_copy_type_inner(value_type, depth + 1)
                            })
                        })
            }
            _ => false,
        }
    }
}

pub(super) fn has_non_lifetime_generic_parameter(parameters: &[GenericParameter]) -> bool {
    parameters
        .iter()
        .any(|parameter| !matches!(parameter, GenericParameter::Lifetime { .. }))
}

pub(super) fn declared_reference_lifetime(value_type: Option<&Type>) -> Option<Lifetime> {
    match value_type {
        Some(Type::Reference { lifetime, .. }) => lifetime.clone(),
        _ => None,
    }
}
