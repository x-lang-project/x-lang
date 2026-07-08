use super::Attempt;
use crate::integer_suffix::IntegerSuffix;

#[derive(Clone, Copy)]
pub(super) struct Token<'a> {
    pub(super) text: &'a str,
    pub(super) start: usize,
    pub(super) end: usize,
}

fn digits(bytes: &[u8], at: &mut usize, radix: u32) -> Attempt<()> {
    fn valid(byte: u8, radix: u32) -> bool {
        (byte as char).is_digit(radix)
    }
    if !bytes
        .get(*at)
        .is_some_and(|second_value| valid(*second_value, radix))
    {
        return None;
    }

    *at += 1;
    loop {
        if bytes
            .get(*at)
            .is_some_and(|second_value| valid(*second_value, radix))
        {
            *at += 1;
        } else if bytes.get(*at) == Some(&b'_') {
            *at += 1;
            if !bytes
                .get(*at)
                .is_some_and(|second_value| valid(*second_value, radix))
            {
                return None;
            }

            *at += 1;
        } else {
            break;
        }
    }

    Some(())
}

fn scan_number(input: &str, start: usize) -> Attempt<usize> {
    let bytes = input.as_bytes();
    let mut at = start;
    let mut floating = false;

    if input
        .get(start..)
        .is_some_and(|text| text.starts_with("0x"))
    {
        at += 2;
        digits(bytes, &mut at, 16)?;
    } else if input
        .get(start..)
        .is_some_and(|text| text.starts_with("0b"))
    {
        at += 2;
        digits(bytes, &mut at, 2)?;
    } else {
        digits(bytes, &mut at, 10)?;
        if bytes.get(at) == Some(&b'.') && bytes.get(at + 1).is_some_and(u8::is_ascii_digit) {
            floating = true;
            at += 1;
            digits(bytes, &mut at, 10)?;
        }

        if bytes
            .get(at)
            .is_some_and(|byte| *byte == b'e' || *byte == b'E')
        {
            floating = true;
            at += 1;
            if bytes
                .get(at)
                .is_some_and(|byte| *byte == b'+' || *byte == b'-')
            {
                at += 1;
            }

            digits(bytes, &mut at, 10)?;
        }
    }

    if !floating && let Some(suffix) = input.get(at..).and_then(IntegerSuffix::starts_token) {
        at += suffix.spelling().len();
    }

    if bytes
        .get(at)
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
    {
        return None;
    }

    Some(at)
}
