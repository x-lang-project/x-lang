use std::iter;

use unicode_width::UnicodeWidthStr;

use crate::FileRecord;

pub(super) fn byte_to_column(file: &FileRecord, line: usize, byte: usize) -> usize {
    let Some(line_start) = line
        .checked_sub(1)
        .and_then(|index| file.line_offsets.get(index))
        .copied()
    else {
        return 1;
    };

    let mut end = byte.clamp(line_start, file.source.len());

    while end > line_start && !file.source.is_char_boundary(end) {
        end -= 1;
    }

    let slice = file.source.get(line_start..end).unwrap_or("");

    UnicodeWidthStr::width(expand_tabs(slice).as_str()) + 1
}

// Source frames use four-column tab stops.
pub(super) fn expand_tabs(text: &str) -> String {
    let mut expanded = String::with_capacity(text.len());
    let mut column = 0;

    for (index, segment) in text.split('\t').enumerate() {
        if index != 0 {
            let spaces = 4 - column % 4;
            expanded.extend(iter::repeat_n(' ', spaces));
            column += spaces;
        }

        expanded.push_str(segment);
        column += UnicodeWidthStr::width(segment);
    }

    expanded
}
