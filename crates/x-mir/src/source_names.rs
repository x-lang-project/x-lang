use x_ast::Type;

pub(crate) fn method_call_function_name(receiver_type: &Type, method: &str) -> String {
    match receiver_type {
        Type::Reference { inner, .. } => method_call_function_name(inner, method),
        Type::RawPointer { .. } => method.to_string(),
        Type::Custom(name) => format!("{name}::{method}"),
        Type::GenericInstance { .. } => {
            format!("{}::{method}", type_to_source_name(receiver_type))
        }
        _ => method.to_string(),
    }
}

pub(crate) fn type_to_source_name(value_type: &Type) -> String {
    value_type.to_string()
}

pub(crate) fn normalise_function_name(name: &str) -> String {
    if let Some(decoded) = decode_x_mangled_function_leaf(name) {
        return decoded;
    }

    strip_generic_segments(name)
}

pub(crate) fn decode_x_mangled_function_leaf(name: &str) -> Option<String> {
    let rest = name.strip_prefix("_XF")?;
    let digit_count = rest.bytes().take_while(u8::is_ascii_digit).count();
    let (length_digits, encoded_name) = rest.split_at(digit_count);
    let name_length = length_digits.parse::<usize>().ok()?;

    encoded_name.get(..name_length).map(str::to_string)
}

pub(crate) fn strip_generic_segments(name: &str) -> String {
    let mut output = String::with_capacity(name.len());
    let mut depth = 0_i32;

    for ch in name.chars() {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            _ if depth == 0 => output.push(ch),
            _ => {}
        }
    }

    output
}
