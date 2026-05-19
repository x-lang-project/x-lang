use std::iter;

pub(super) fn first_backtick_before(text: &str, marker: &str) -> Option<String> {
    let marker_index = text.find(marker)?;
    let before = &text[..marker_index];

    backtick_values(before)
        .next()
        .filter(|name| is_source_lookup_candidate(name))
        .map(str::to_string)
}

pub(super) fn second_backtick_between(text: &str, marker: &str) -> Option<String> {
    text.find(marker)?;

    backtick_values(text)
        .nth(1)
        .filter(|name| is_source_lookup_candidate(name))
        .map(str::to_string)
}

pub(super) fn is_source_lookup_candidate(name: &str) -> bool {
    !name.is_empty() && !name.contains(char::is_whitespace)
}

pub(super) fn backtick_values(mut text: &str) -> impl Iterator<Item = &str> {
    iter::from_fn(move || {
        let (_, after_open) = text.split_once('`')?;
        let (value, after_close) = after_open.split_once('`')?;
        text = after_close;

        Some(value)
    })
    .filter(|value| !value.is_empty())
}
