#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreMemoryOperation {
    CopyObjectsNonoverlapping,
    CopyBytesNonoverlapping,
    CopyBytes,
    MoveObjects,
    MoveBytes,
    SetBytes,
    ZeroObjects,
    ZeroBytes,
    Fill,
    CheckedArraySize,
}

impl CoreMemoryOperation {
    pub fn resolve(leaf: &str) -> Option<Self> {
        Some(match leaf {
            "copy_nonoverlapping" => Self::CopyObjectsNonoverlapping,
            "copy_nonoverlapping_bytes" => Self::CopyBytesNonoverlapping,
            "copy" => Self::CopyBytes,
            "move_objects" => Self::MoveObjects,
            "move_bytes" => Self::MoveBytes,
            "set_bytes" => Self::SetBytes,
            "zero_objects" => Self::ZeroObjects,
            "zero_bytes" => Self::ZeroBytes,
            "fill" => Self::Fill,
            "checked_array_size" => Self::CheckedArraySize,
            _ => return None,
        })
    }

    pub const fn source_name(self) -> &'static str {
        match self {
            Self::CopyObjectsNonoverlapping => "copy_nonoverlapping",
            Self::CopyBytesNonoverlapping => "copy_nonoverlapping_bytes",
            Self::CopyBytes => "copy",
            Self::MoveObjects => "move_objects",
            Self::MoveBytes => "move_bytes",
            Self::SetBytes => "set_bytes",
            Self::ZeroObjects => "zero_objects",
            Self::ZeroBytes => "zero_bytes",
            Self::Fill => "fill",
            Self::CheckedArraySize => "checked_array_size",
        }
    }
}
