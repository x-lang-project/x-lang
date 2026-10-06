use super::*;

pub(super) fn width(value_type: &Type) -> Option<u32> {
    Some(match value_type {
        Type::I8 | Type::U8 | Type::Character => 8,
        Type::I16 | Type::U16 => 16,
        Type::I32 | Type::U32 => 32,
        Type::I64 | Type::U64 | Type::ISize | Type::USize => 64,
        Type::Boolean => 1,
        _ => return None,
    })
}

pub(super) fn signed(value_type: &Type) -> bool {
    matches!(
        value_type,
        Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::ISize
    )
}

pub(super) fn uint(writer: u32) -> &'static str {
    match writer {
        1 | 8 => "unsigned char",
        16 => "unsigned short",
        32 => "unsigned int",
        _ => "unsigned long long",
    }
}

pub(super) fn sint(writer: u32) -> &'static str {
    match writer {
        1 | 8 => "signed char",
        16 => "short",
        32 => "int",
        _ => "long long",
    }
}

pub(super) fn encoded(name: &str) -> String {
    name.as_bytes()
        .iter()
        .map(|value| format!("{value:02x}"))
        .collect()
}

pub(super) fn ctype(value_type: &Type) -> Result<String> {
    Ok(match value_type {
        Type::Void => "void".into(),
        Type::F32 => "float".into(),
        Type::F64 => "double".into(),
        Type::String => "struct _x_str *".into(),
        Type::RawPointer { inner, .. } | Type::Reference { inner, .. } => {
            format!("{} *", ctype(inner)?)
        }
        Type::Custom(name) => format!("struct _x_t{}", encoded(name)),
        Type::FixedSizedArray(_, _) | Type::Tuple(_) => {
            format!("struct _x_a{}", encoded(&format!("{value_type:?}")))
        }
        value_type => uint(
            width(value_type)
                .ok_or_else(|| backend_messages::c_emission_unsupported_type(value_type))?,
        )
        .into(),
    })
}

pub(super) fn pointer(value_type: &Type) -> bool {
    matches!(
        value_type,
        Type::String | Type::RawPointer { .. } | Type::Reference { .. }
    )
}

pub(super) fn bits(expression: &str, writer: u32) -> String {
    if writer == 1 {
        format!("((unsigned char)(({expression})&1))")
    } else {
        format!("(({})({expression}))", uint(writer))
    }
}

pub(super) fn signed_view(value: &Value) -> Result<String> {
    Ok(format!(
        "(({})({}))",
        sint(width(&value.value_type).ok_or_else(|| EmitError::internal(
            backend_messages::c_emission_numeric_value_width_is_missing()
        ))?),
        value.code
    ))
}

pub(super) fn convert(value: Value, target: &Type, sign_extend: bool) -> Result<Value> {
    if &value.value_type == target {
        return Ok(value);
    }

    let code = match (width(&value.value_type), width(target)) {
        _ => {
            return Err(backend_messages::c_emission_unsupported_conversion(
                value.value_type,
                target,
            )
            .into());
        }
    };

    Ok(Value {
        code,
        value_type: target.clone(),
    })
}
