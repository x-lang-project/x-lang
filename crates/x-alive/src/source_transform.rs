use std::collections::HashSet;

use x_diagnostics::alive_messages;

fn protected_end(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start..start + 2) == Some(b"//") {
        return Some(
            start
                + bytes
                    .get(start..)?
                    .iter()
                    .position(|second_value| *second_value == b'\n')
                    .unwrap_or(bytes.len() - start),
        );
    }

    if bytes.get(start..start + 2) == Some(b"/*") {
        return Some(
            start
                + 2
                + bytes
                    .get(start + 2..)?
                    .windows(2)
                    .position(|second_value| second_value == b"*/")
                    .map_or(bytes.len() - start - 2, |count| count + 2),
        );
    }

    if bytes.get(start) == Some(&b'\'')
        && let Some(end) = character_literal_end(bytes, start)
    {
        return Some(end);
    }

    if bytes.get(start) != Some(&b'"') {
        return None;
    }

    let mut end = start + 1;

    while let Some(byte) = bytes.get(end) {
        match byte {
            b'\\' => end += 2,
            b'"' => return Some(end + 1),
            _ => end += 1,
        }
    }

    Some(bytes.len())
}

// A lifetime quote must not be consumed as a character literal.
fn character_literal_end(bytes: &[u8], start: usize) -> Option<usize> {
    let value_start = start.checked_add(1)?;
    let value_end = if bytes.get(value_start) == Some(&b'\\') {
        let escaped = value_start.checked_add(1)?;

        if bytes.get(escaped..escaped.checked_add(2)?) == Some(b"u{") {
            let digits = escaped.checked_add(2)?;
            let count = bytes
                .get(digits..)?
                .iter()
                .take_while(|byte| byte.is_ascii_hexdigit())
                .count();
            let close = digits.checked_add(count)?;

            if count == 0 || bytes.get(close) != Some(&b'}') {
                return None;
            }

            close.checked_add(1)?
        } else {
            scalar_end(bytes, escaped)?
        }
    } else {
        scalar_end(bytes, value_start)?
    };

    (bytes.get(value_end) == Some(&b'\'')).then(|| value_end + 1)
}

fn scalar_end(bytes: &[u8], start: usize) -> Option<usize> {
    let width = match bytes.get(start)? {
        0x00..=0x7f => 1,
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => return None,
    };

    let end = start.checked_add(width)?;
    std::str::from_utf8(bytes.get(start..end)?).ok()?;

    Some(end)
}

fn balanced_end(
    bytes: &[u8],
    start: usize,
    open: u8,
    close: u8,
) -> Result<usize, alive_messages::AliveError> {
    let mut depth = 0;
    let mut cursor = start;

    while let Some(&byte) = bytes.get(cursor) {
        if let Some(end) = protected_end(bytes, cursor) {
            cursor = end;
            continue;
        }

        if byte == open {
            depth += 1;
        } else if byte == close {
            depth -= 1;

            if depth == 0 {
                return Ok(cursor + 1);
            }
        }
        cursor += 1;
    }

    Err(alive_messages::unclosed_block(start))
}

fn skip_whitespace(bytes: &[u8], mut cursor: usize) -> usize {
    while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
        cursor += 1;
    }

    cursor
}

fn identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || !byte.is_ascii()
}

pub(super) fn strip_keywords(
    source: &str,
    keywords: &[String],
) -> Result<String, alive_messages::AliveError> {
    if keywords.is_empty() {
        return Ok(source.to_string());
    }

    let mut keywords: HashSet<String> = keywords
        .iter()
        .map(|text| text.to_ascii_lowercase())
        .collect();

    if keywords.iter().any(|keyword| {
        matches!(
            keyword.as_str(),
            "memo" | "memoise" | "memoize" | "memoised" | "memoized"
        )
    }) {
        keywords.extend(["memo", "memoise", "memoize", "memoised", "memoized"].map(String::from));
    }

    let bytes = source.as_bytes();
    let mut output = bytes.to_vec();
    let mut cursor = 0;

    while let Some(&byte) = bytes.get(cursor) {
        if let Some(end) = protected_end(bytes, cursor) {
            cursor = end;
            continue;
        }

        let start = cursor;
        let mut remove = false;

        if identifier_byte(byte) {
            cursor += 1;
            while bytes
                .get(cursor)
                .is_some_and(|second_value| identifier_byte(*second_value))
            {
                cursor += 1;
            }

            let token = source
                .get(start..cursor)
                .ok_or_else(alive_messages::invalid_token_boundary)?
                .to_ascii_lowercase();
            remove = keywords.contains(&token);

            if remove && token == "invariant" {
                let brace = skip_whitespace(bytes, cursor);

                if bytes.get(brace) == Some(&b'{') {
                    cursor = balanced_end(bytes, brace, b'{', b'}')?;
                }
            }
        } else if byte == b'#' {
            let bracket = skip_whitespace(bytes, cursor + 1);

            if bytes.get(bracket) == Some(&b'[') {
                let end = balanced_end(bytes, bracket, b'[', b']')?;
                let contents = source
                    .get(bracket + 1..end - 1)
                    .ok_or_else(alive_messages::invalid_attribute_boundary)?
                    .trim();
                let name_end = contents
                    .bytes()
                    .position(|byte| !identifier_byte(byte))
                    .unwrap_or(contents.len());
                let name = contents
                    .get(..name_end)
                    .ok_or_else(alive_messages::invalid_attribute_name_boundary)?
                    .to_ascii_lowercase();
                remove = keywords.contains(&name)
                    || contents.is_empty()
                    || contents
                        .strip_prefix("Layout")
                        .is_some_and(|rest| rest.trim_start().starts_with('('));
                cursor = end;
            } else {
                cursor += 1;
            }
        } else {
            cursor += 1;
        }

        if remove {
            for byte in output
                .get_mut(start..cursor)
                .ok_or_else(alive_messages::invalid_edit_boundary)?
            {
                if !byte.is_ascii_whitespace() {
                    *byte = b' ';
                }
            }
        }
    }

    String::from_utf8(output).map_err(alive_messages::invalid_transformed_source)
}
