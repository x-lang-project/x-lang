use crate::Type;

pub fn type_symbol_component(value_type: &Type) -> String {
    match value_type {
        Type::I8
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
        | Type::F64
        | Type::Boolean
        | Type::String
        | Type::Character
        | Type::Void
        | Type::Unknown => value_type.to_string(),
        Type::Never => "never".into(),
        Type::Infer(identifier) => format!("I{identifier}_"),
        Type::Custom(name) => format!("N{}", symbol_name_component(name)),
        Type::TypeParameter(name) => format!("Q{}", symbol_name_component(name)),
        Type::Array(inner) => format!("A{}", framed_type(inner)),
        Type::FixedSizedArray(inner, count) => format!("F{count}_{}", framed_type(inner)),
        Type::Simd { element, lanes } => format!("V{lanes}_{}", framed_type(element)),
        Type::Tuple(items) => format!("T{}", framed_types(items)),
        Type::RawPointer { is_mut, inner } => {
            format!("P{}{}", u8::from(*is_mut), framed_type(inner))
        }
        Type::Reference {
            is_mut,
            is_unique,
            inner,
            ..
        } => format!(
            "R{}{}{}",
            u8::from(*is_unique),
            u8::from(*is_mut),
            framed_type(inner)
        ),
        Type::GenericInstance {
            name,
            type_arguments,
        } => {
            format!(
                "G{}{}",
                symbol_name_component(name),
                framed_types(type_arguments)
            )
        }
        Type::Function {
            parameters,
            return_value,
        } => format!(
            "Fn{}{}",
            framed_types(parameters),
            framed_type(return_value)
        ),
    }
}

pub fn type_symbol_components(types: &[Type]) -> String {
    types
        .iter()
        .map(type_symbol_component)
        .collect::<Vec<_>>()
        .join("_")
}

fn framed_type(value_type: &Type) -> String {
    let encoded = type_symbol_component(value_type);

    format!("{}_{encoded}", encoded.len())
}

fn framed_types(types: &[Type]) -> String {
    let encoded = types.iter().map(framed_type).collect::<String>();

    format!("{}_{encoded}", types.len())
}

pub fn symbol_name_component(name: &str) -> String {
    let encoded = name
        .bytes()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    format!("{}_{encoded}", name.len())
}
