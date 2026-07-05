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
