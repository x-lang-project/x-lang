fn function_leaf_start(function: &str) -> usize {
    let bytes = function.as_bytes();
    let mut depth: usize = 0;

    for (index, byte) in bytes.iter().copied().enumerate().rev() {
        match byte {
            b'>' => depth += 1,
            b'<' => depth = depth.saturating_sub(1),
            b':' if depth == 0
                && index > 0
                && bytes.get(index - 1) == Some(&b':')
                && bytes.get(index + 1) != Some(&b'<') =>
            {
                return index + 1;
            }
            _ => {}
        }
    }

    0
}

pub fn function_leaf_name(function: &str) -> &str {
    let leaf = &function[function_leaf_start(function)..];

    leaf.split('<')
        .next()
        .unwrap_or(leaf)
        .trim_end_matches("::")
}

pub fn function_owner_name(function: &str) -> Option<&str> {
    let mut depth: usize = 0;

    for byte in function.bytes() {
        match byte {
            b'<' => depth += 1,
            b'>' => depth = depth.checked_sub(1)?,
            _ => {}
        }
    }

    if depth != 0 {
        return None;
    }

    let leaf_start = function_leaf_start(function);

    if leaf_start == function.len() {
        return None;
    }

    let owner = function.get(..leaf_start.checked_sub(2)?)?;

    (!owner.is_empty()).then_some(owner)
}

pub fn parse_generic_type_argument(function: &str) -> Option<&str> {
    let (start, end) = generic_argument_bounds(function)?;
    let value_type = function[start + 1..end].trim();

    (!value_type.is_empty()).then_some(value_type)
}

pub fn strip_explicit_type_arguments(name: &str) -> &str {
    match generic_argument_bounds(name) {
        Some((start, end)) if end + 1 == name.len() => name[..start].trim_end_matches("::"),
        _ => name,
    }
}

fn generic_argument_bounds(name: &str) -> Option<(usize, usize)> {
    let start = name.find('<')?;
    let mut depth: usize = 0;

    for (offset, character) in name[start..].char_indices() {
        match character {
            '<' => depth += 1,
            '>' => {
                depth = depth.saturating_sub(1);

                if depth == 0 {
                    return Some((start, start + offset));
                }
            }
            _ => {}
        }
    }

    None
}

pub fn is_bit_intrinsic_call(leaf: &str) -> bool {
    crate::BitIntrinsic::resolve(leaf).is_some()
}

pub fn is_heap_allocate_call(base: &str, leaf: &str) -> bool {
    matches!(leaf, "alloc" | "alloc_zeroed") && base.strip_prefix("heap::") == Some(leaf)
}

pub fn is_mem_namespace_call(base: &str, leaf: &str, expected_leaf: &str) -> bool {
    leaf == expected_leaf && base.strip_prefix("mem::") == Some(expected_leaf)
}

pub fn is_pointer_namespace_named_call(base: &str, leaf: &str, expected_leaf: &str) -> bool {
    leaf == expected_leaf && base.strip_prefix("ptr::") == Some(expected_leaf)
}

pub fn is_abort_call(full: &str) -> bool {
    matches!(
        full,
        "abort" | "trap" | "panic::abort" | "panic::trap" | "intrinsic::process_abort"
    ) || full == "process_abort"
}

pub fn is_builtin_print_call(function: &str, name: &str) -> bool {
    let Some(leaf) = function
        .strip_prefix("io::")
        .or_else(|| function.strip_prefix("std::io::"))
    else {
        return false;
    };

    leaf == name
        || leaf
            .strip_prefix(name)
            .and_then(|suffix| suffix.strip_prefix("__multi_"))
            .and_then(crate::Type::from_primitive_name)
            .is_some_and(|value_type| {
                value_type.is_scalar()
                    || matches!(value_type, crate::Type::Character | crate::Type::String)
            })
}

pub fn is_catch_call(name: &str) -> bool {
    name == "catch"
}
