use super::{CallPath, CallSegment};

pub(super) fn decode_path(name: &str) -> CallPath {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut depth: usize = 0;
    let mut chars = name.char_indices().peekable();

    while let Some((offset, character)) = chars.next() {
        match character {
            '<' | '(' | '[' => depth += 1,
            '>' | ')' | ']' => depth = depth.saturating_sub(1),
            ':' if depth == 0 && chars.peek().is_some_and(|(_, ch)| *ch == ':') => {
                chars.next();
                if chars.peek().is_some_and(|(_, ch)| *ch == '<') {
                    continue;
                }

                if let Some(part) = name.get(start..offset) {
                    segments.push(decode_segment(part));
                }
                start = offset + 2;
            }
            _ => {}
        }
    }

    if let Some(part) = name.get(start..) {
        segments.push(decode_segment(part));
    }

    CallPath { segments }
}

fn decode_segment(segment: &str) -> CallSegment {
    let Some((name, current_arguments)) = segment.split_once('<') else {
        return CallSegment {
            name: segment.trim().into(),
            type_arguments: Vec::new(),
        };
    };

    let Some(current_arguments) = current_arguments.strip_suffix('>') else {
        return CallSegment {
            name: segment.trim().into(),
            type_arguments: Vec::new(),
        };
    };

    let arguments = split_argument_names(current_arguments)
        .iter()
        .map(|argument| crate::type_from_name(argument))
        .collect();

    CallSegment {
        name: name.trim().trim_end_matches("::").into(),
        type_arguments: arguments,
    }
}

fn split_argument_names(arguments: &str) -> Vec<String> {
    let mut depth: usize = 0;
    let mut start = 0;
    let mut parts = Vec::new();

    for (offset, character) in arguments.char_indices() {
        match character {
            '<' | '(' | '[' => depth += 1,
            '>' | ')' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                if let Some(part) = arguments.get(start..offset) {
                    parts.push(part.trim().to_owned());
                }
                start = offset + 1;
            }
            _ => {}
        }
    }

    if let Some(part) = arguments
        .get(start..)
        .filter(|part| !part.trim().is_empty())
    {
        parts.push(part.trim().to_owned());
    }

    parts
}
