pub(super) fn find_identifier(source: &str, name: &str) -> Option<usize> {
    if name.is_empty() {
        return None;
    }

    let mut first = None;
    let mut first_non_declaration = None;

    for (start, _) in source.match_indices(name) {
        let end = start + name.len();
        let before = source[..start].chars().next_back();
        let after = source[end..].chars().next();

        if before.is_some_and(is_ident_char) || after.is_some_and(is_ident_char) {
            continue;
        }

        first.get_or_insert(start);

        if !is_function_declaration(source, start) {
            if is_call_site(source, start, name) {
                return Some(start);
            }

            first_non_declaration.get_or_insert(start);
        }
    }

    first_non_declaration.or(first)
}

fn is_call_site(source: &str, start: usize, name: &str) -> bool {
    source[start + name.len()..]
        .chars()
        .find(|ch| !ch.is_whitespace())
        == Some('(')
}

fn is_function_declaration(source: &str, start: usize) -> bool {
    let line_start = source[..start]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);

    source[line_start..start].split_whitespace().next_back() == Some("fn")
}

fn find_in_source_lines(
    source: &str,
    mut find: impl FnMut(&str) -> Option<usize>,
) -> Option<usize> {
    let mut offset = 0;

    for line in source.split_inclusive('\n') {
        if let Some(column) = find(line) {
            return Some(offset + column);
        }

        offset += line.len();
    }

    None
}

pub(super) fn find_mutating_assignment(source: &str) -> Option<usize> {
    find_in_source_lines(source, |line| {
        let trimmed = line.trim_start();

        if trimmed.starts_with("let ") || trimmed.starts_with("const ") {
            None
        } else {
            line.find('=')
        }
    })
}

fn is_ident_char(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}
