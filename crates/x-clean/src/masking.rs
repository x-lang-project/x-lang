//! Mask comments and strings without changing byte offsets.
pub fn mask(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut result = bytes.to_vec();
    let mut index = 0;

    while index < bytes.len() {
        let start = index;

        if bytes[index..].starts_with(b"//") {
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
        } else if bytes[index..].starts_with(b"/*") {
            index += 2;
            let mut depth = 1;

            while index < bytes.len() && depth > 0 {
                if bytes[index..].starts_with(b"/*") {
                    depth += 1;
                    index += 2;
                } else if bytes[index..].starts_with(b"*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    index += 1;
                }
            }
        } else if let Some(end) = raw_end(bytes, index) {
            index = end;
        } else if bytes[index] == b'"' {
            index += 1;

            while index < bytes.len() {
                if bytes[index] == b'\\' {
                    index = (index + 2).min(bytes.len());
                } else if bytes[index] == b'"' {
                    index += 1;
                    break;
                } else {
                    index += 1;
                }
            }
        } else if bytes[index] == b'\''
            && let Some(end) = char_end(text, index)
        {
            index = end;
        } else {
            index += 1;
            continue;
        }

        for byte in &mut result[start..index] {
            if !matches!(*byte, b'\r' | b'\n') {
                *byte = b' ';
            }
        }
    }

    String::from_utf8(result).expect("masking only blanks complete literal/comment bytes")
}

fn char_end(text: &str, start: usize) -> Option<usize> {
    let tail = &text[start + 1..];
    let mut chars = tail.char_indices();
    let (_, first) = chars.next()?;

    if first == '\\' {
        for (index, character) in chars.take(16) {
            if character == '\'' {
                return Some(start + index + 2);
            }

            if character == '\n' {
                break;
            }
        }

        None
    } else {
        let (index, character) = chars.next()?;

        (character == '\'').then_some(start + index + 2)
    }
}

fn raw_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut index = start;

    if matches!(bytes[index], b'b' | b'c') {
        index += 1;
    }

    if bytes.get(index) != Some(&b'r') {
        return None;
    }

    index += 1;
    let hashes = index;

    while bytes.get(index) == Some(&b'#') {
        index += 1;
    }

    let count = index - hashes;

    if bytes.get(index) != Some(&b'"') {
        return None;
    }

    index += 1;
    while index < bytes.len() {
        if bytes[index] == b'"'
            && bytes
                .get(index + 1..index + 1 + count)
                .is_some_and(|part| part.iter().all(|byte| *byte == b'#'))
        {
            return Some(index + 1 + count);
        }

        index += 1;
    }

    Some(bytes.len())
}
