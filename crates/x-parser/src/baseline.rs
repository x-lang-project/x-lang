use x_ast::{Layout, Program, Statement, Type, TypeVisitorMut, walk_statement_types_mut};

struct Baseline;

fn strip_semantic_features_in_type(value_type: &mut Type) {
    match value_type {
        Type::Array(inner)
        | Type::FixedSizedArray(inner, _)
        | Type::RawPointer { inner, .. }
        | Type::Simd { element: inner, .. } => strip_semantic_features_in_type(inner),
        Type::GenericInstance {
            type_arguments: items,
            ..
        }
        | Type::Tuple(items) => {
            for item in items {
                strip_semantic_features_in_type(item);
            }
        }
        Type::Reference {
            is_unique, inner, ..
        } => {
            *is_unique = false;
            strip_semantic_features_in_type(inner);
        }
        Type::Function {
            parameters,
            return_value,
        } => {
            for parameter in parameters {
                strip_semantic_features_in_type(parameter);
            }

            strip_semantic_features_in_type(return_value);
        }
        Type::I32
        | Type::F32
        | Type::String
        | Type::Character
        | Type::Boolean
        | Type::Void
        | Type::Never
        | Type::Custom(_)
        | Type::Unknown
        | Type::Infer(_)
        | Type::TypeParameter(_)
        | Type::I8
        | Type::U8
        | Type::I16
        | Type::U16
        | Type::U32
        | Type::I64
        | Type::U64
        | Type::ISize
        | Type::USize
        | Type::F64 => {}
    }
}
