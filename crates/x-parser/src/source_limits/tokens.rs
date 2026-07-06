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
