use super::*;

pub(super) fn require_scanner_progress(
    current: usize,
    next: usize,
) -> Result<(), SourceLimitError> {
    if next > current {
        Ok(())
    } else {
        Err(SourceLimitError::ScannerDidNotAdvance { offset: current })
    }
}

pub(super) fn scan_line_comment(bytes: &[u8], start: usize) -> usize {
    bytes
        .get(start..)
        .unwrap_or_default()
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(bytes.len(), |relative| start.saturating_add(relative))
}

pub(super) fn scan_quoted_literal(bytes: &[u8], start: usize) -> Result<usize, SourceLimitError> {
    let Some(&quote) = bytes.get(start) else {
        return Err(SourceLimitError::ScannerDidNotAdvance { offset: start });
    };

    let mut escaped = false;

    for (relative, byte) in bytes
        .get(start.saturating_add(1)..)
        .unwrap_or_default()
        .iter()
        .copied()
        .enumerate()
    {
        let index = start.saturating_add(1).saturating_add(relative);

        if !escaped && byte == quote {
            return Ok(index.saturating_add(1));
        }

        escaped = byte == b'\\' && !escaped;

        if quote == b'"' && index.saturating_add(1).saturating_sub(start) > MAX_STRING_LITERAL_BYTES
        {
            return Err(SourceLimitError::StringLiteralTooLarge {
                offset: start,
                maximum: MAX_STRING_LITERAL_BYTES,
            });
        }
    }

    Ok(bytes.len())
}

pub(super) fn record_token(
    token_count: &mut usize,
    offset: usize,
    maximum: usize,
) -> Result<(), SourceLimitError> {
    *token_count += 1;

    if *token_count > maximum {
        Err(SourceLimitError::TooManyTokens { offset, maximum })
    } else {
        Ok(())
    }
}

pub(super) fn validate_numeric_literal(
    literal: &str,
    offset: usize,
) -> Result<(), SourceLimitError> {
    let normalized = if literal.contains('_') {
        Cow::Owned(literal.replace('_', ""))
    } else {
        Cow::Borrowed(literal)
    };

    if normalized.contains(['.', 'e', 'E']) {
        if normalized
            .parse::<f64>()
            .is_ok_and(|value| !value.is_finite())
        {
            return Err(SourceLimitError::NonFiniteFloat { offset });
        }

        return Ok(());
    }

    let (digits, _) = crate::integer_suffix::IntegerSuffix::split(&normalized);

    if !digits.is_empty()
        && digits.bytes().all(|byte| byte.is_ascii_digit())
        && digits.parse::<i64>().is_err()
    {
        return Err(SourceLimitError::IntegerOutOfRange { offset });
    }

    Ok(())
}
