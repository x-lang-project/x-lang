use crate::errors::CodeGenerationError;
use crate::symbols::{SymbolKind, SymbolTable};
use x_ast::Type;

fn generate_c_header(table: &SymbolTable) -> Result<String, CodeGenerationError> {
    let mut output = String::from("#pragma once\n#include <stdbool.h>\n#include <stdint.h>\n\n");

    for symbol in table.symbols.iter().filter(|symbol| symbol.is_exported) {
        match symbol.kind {
            SymbolKind::Function => {
                let return_value = symbol
                    .return_type
                    .as_ref()
                    .map(c_type)
                    .ok_or(CodeGenerationError::UnsupportedType(Type::Unknown))??;
                let parameters = if symbol.parameters.is_empty() {
                    "void".to_string()
                } else {
                    symbol
                        .parameters
                        .iter()
                        .enumerate()
                        .map(|(index, value_type)| {
                            c_type(value_type).map(|value_type| format!("{value_type} arg{index}"))
                        })
                        .collect::<Result<Vec<_>, _>>()?
                        .join(", ")
                };

                output.push_str(&format!(
                    "extern {return_value} {}({parameters});\n",
                    symbol.symbol_name
                ));
            }
            SymbolKind::Static => {
                let value_type = symbol
                    .return_type
                    .as_ref()
                    .map(c_type)
                    .ok_or(CodeGenerationError::UnsupportedType(Type::Unknown))??;
                output.push_str(&format!("extern {value_type} {};\n", symbol.symbol_name));
            }
        }
    }

    Ok(output)
}

fn c_type(value_type: &Type) -> Result<String, CodeGenerationError> {
    Ok(match value_type {
        Type::I8 => "int8_t",
        Type::U8 | Type::Character => "uint8_t",
        Type::I16 => "int16_t",
        Type::U16 => "uint16_t",
        Type::I32 => "int32_t",
        Type::U32 => "uint32_t",
        Type::I64 => "int64_t",
        Type::ISize => "intptr_t",
        Type::U64 => "uint64_t",
        Type::USize => "uintptr_t",
        Type::F32 => "float",
        Type::F64 => "double",
        Type::Boolean => "bool",
        Type::Void => "void",
        Type::RawPointer { is_mut, inner } | Type::Reference { is_mut, inner, .. } => {
            let qualifier = if *is_mut { "" } else { "const " };

            return Ok(format!("{qualifier}{}*", c_type(inner)?));
        }
        Type::FixedSizedArray(_, _) => {
            return Err(CodeGenerationError::UnsupportedType(value_type.clone()));
        }
        Type::Never
        | Type::String
        | Type::Array(_)
        | Type::Function { .. }
        | Type::Simd { .. }
        | Type::Tuple(_)
        | Type::Custom(_)
        | Type::GenericInstance { .. }
        | Type::Unknown
        | Type::Infer(_)
        | Type::TypeParameter(_) => {
            return Err(CodeGenerationError::UnsupportedType(value_type.clone()));
        }
    }
    .to_string())
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            other => escaped.push(other),
        }
    }

    escaped
}
