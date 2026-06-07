use super::*;

pub fn unclosed_block(start: impl Display) -> AliveError {
    AliveError::new(
        AliveFailureKind::UnclosedBlock,
        Error::new(format!("Unclosed comparison block at byte {start}")),
    )
}
