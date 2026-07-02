use std::fmt;

use crate::{ExpressionKind, Lifetime, StringPart, Type};

impl fmt::Display for StringPart {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StringPart::Text(text) => write!(formatter, "{text}"),
            StringPart::Interpolation(expression) => match &expression.kind {
                ExpressionKind::String(str_literal) => {
                    for part in &str_literal.parts {
                        write!(formatter, "{part}")?;
                    }

                    Ok(())
                }
                ExpressionKind::Identifier(name) => write!(formatter, "{name}"),
                _ => write!(formatter, "{expression:?}"),
            },
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::I8 => write!(formatter, "i8"),
            Type::U8 => write!(formatter, "u8"),
            Type::I16 => write!(formatter, "i16"),
            Type::U16 => write!(formatter, "u16"),
            Type::I32 => write!(formatter, "i32"),
            Type::U32 => write!(formatter, "u32"),
            Type::I64 => write!(formatter, "i64"),
            Type::U64 => write!(formatter, "u64"),
            Type::ISize => write!(formatter, "isize"),
            Type::USize => write!(formatter, "usize"),
            Type::F32 => write!(formatter, "f32"),
            Type::F64 => write!(formatter, "f64"),

            Type::Boolean => write!(formatter, "bool"),
            Type::String => write!(formatter, "str"),
            Type::Character => write!(formatter, "char"),
            Type::Void => write!(formatter, "void"),
            Type::Never => write!(formatter, "!"),
            Type::Custom(text) => write!(formatter, "{text}"),
            Type::Infer(identifier) => write!(formatter, "?{identifier}"),
            Type::Unknown => write!(formatter, "unknown"),

            Type::Array(inner) => write!(formatter, "{inner}[]"),
            Type::FixedSizedArray(inner, size) => write!(formatter, "[{inner}; {size}]"),
            Type::Tuple(elements) => write!(formatter, "({})", DisplayTypes(elements)),
            Type::Simd { element, lanes } => write!(formatter, "simd<{element}, {lanes}>"),

            Type::TypeParameter(name) => write!(formatter, "{name}"),
            Type::GenericInstance {
                name,
                type_arguments,
            } => {
                write!(formatter, "{name}<{}>", DisplayTypes(type_arguments))
            }
            Type::Function {
                parameters,
                return_value,
            } => {
                write!(
                    formatter,
                    "fn({}) -> {return_value}",
                    DisplayTypes(parameters)
                )
            }
            Type::RawPointer { is_mut, inner } => {
                write!(
                    formatter,
                    "*{} {}",
                    if *is_mut { "mut" } else { "const" },
                    inner
                )
            }
            Type::Reference {
                lifetime,
                is_mut,
                is_unique,
                inner,
            } => {
                write!(
                    formatter,
                    "&{}{}{}{}",
                    if *is_unique { "unique " } else { "" },
                    lifetime
                        .as_ref()
                        .map(|lifetime| format!("{lifetime} "))
                        .unwrap_or_default(),
                    if *is_mut { "mut " } else { "" },
                    inner
                )
            }
        }
    }
}

struct DisplayTypes<'a>(&'a [Type]);

impl fmt::Display for DisplayTypes<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, value_type) in self.0.iter().enumerate() {
            if index != 0 {
                formatter.write_str(", ")?;
            }

            write!(formatter, "{value_type}")?;
        }

        Ok(())
    }
}

impl fmt::Display for Lifetime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Lifetime::Static => write!(formatter, "'static"),
            Lifetime::Named(name) => write!(formatter, "{name}"),
            Lifetime::Infer(identifier) => write!(formatter, "'_{identifier}"),
            Lifetime::Erased => write!(formatter, "'_"),
        }
    }
}
