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
