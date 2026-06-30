use crate::Type;

const MAX_TYPE_NAME_DEPTH: usize = 64;

pub fn primitive_type_from_name(name: &str) -> Option<Type> {
    Type::from_primitive_name(name.trim())
}

pub fn type_from_name(name: &str) -> Type {
    decode_type_name(name, 0)
}

fn decode_type_name(name: &str, depth: usize) -> Type {
    let name = name.trim();

    if let Some(value_type) = primitive_type_from_name(name) {
        return value_type;
    }

    if depth >= MAX_TYPE_NAME_DEPTH {
        return Type::Custom(name.into());
    }

    if let Some((base, arguments)) = generic_type_parts(name) {
        let base = base.trim();

        return Type::GenericInstance {
            name: base.into(),
            type_arguments: arguments
                .into_iter()
                .map(|argument| decode_type_name(argument, depth + 1))
                .collect(),
        };
    }

    Type::Custom(name.into())
}

fn generic_type_parts(name: &str) -> Option<(&str, Vec<&str>)> {
    let (base, tail) = name.split_once('<')?;
    let arguments = tail.strip_suffix('>')?;
    let mut nesting: usize = 0;
    let mut start = 0;
    let mut current_arguments = Vec::new();

    for (offset, ch) in arguments.char_indices() {
        match ch {
            '<' => nesting += 1,
            '>' if nesting > 0 => nesting -= 1,
            '>' => return None,
            ',' if nesting == 0 => {
                current_arguments.push(&arguments[start..offset]);
                start = offset + 1;
            }
            _ => {}
        }
    }

    current_arguments.push(&arguments[start..]);

    (nesting == 0
        && !base.trim().is_empty()
        && current_arguments
            .iter()
            .all(|argument| !argument.trim().is_empty()))
    .then_some((base, current_arguments))
}
