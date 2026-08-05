pub(crate) use x_utils::sanitize_symbol as sanitize_symbol_component;

#[macro_export]
macro_rules! function_signature {
    ($params:expr => $return_type:expr) => {
        $crate::function_signature!($params => $return_type, throws: false, is_static: false)
    };
    ($params:expr => $return_type:expr, throws: $is_throws:expr, is_static: $is_static:expr $(,)?) => {
        $crate::FunctionSignature {
            parameter_types: $params,
            return_type: $return_type,
            is_throws: $is_throws,
            is_pure: false,
            is_static: $is_static,
        }
    };
}

pub(crate) fn split_top_level_commas(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth: usize = 0;
    let mut start = 0;

    for (index, ch) in text.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(&text[start..index]);
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }

    parts.push(&text[start..]);

    parts
}
