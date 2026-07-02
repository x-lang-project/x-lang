#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerOperation {
    IntrinsicOffset,
    Add,
    Subtract,
    Offset,
    AsReference,
    AsMutableReference,
    Read,
    ReadUnaligned,
    ReadVolatile,
    Write,
    WriteUnaligned,
    WriteVolatile,
    Replace,
    Swap,
    Copy,
    CopyNonoverlapping,
    WriteBytes,
    IsNull,
    FromAddress,
    WithAddress,
    Address,
}

impl PointerOperation {
    pub fn resolve(name: &str) -> Option<Self> {
        Some(match name {
            "ptr_offset" | "ptr_offset_mut" => Self::IntrinsicOffset,
            "add" | "add_mut" => Self::Add,
            "sub" | "sub_mut" => Self::Subtract,
            "offset" | "offset_mut" => Self::Offset,
            "as_ref" => Self::AsReference,
            "as_mut" => Self::AsMutableReference,
            "read" | "ptr_read" => Self::Read,
            "read_unaligned" => Self::ReadUnaligned,
            "read_volatile" | "volatile_read" => Self::ReadVolatile,
            "write" | "ptr_write" => Self::Write,
            "write_unaligned" => Self::WriteUnaligned,
            "write_volatile" | "volatile_write" => Self::WriteVolatile,
            "replace" => Self::Replace,
            "swap" => Self::Swap,
            "copy" => Self::Copy,
            "copy_nonoverlapping" => Self::CopyNonoverlapping,
            "write_bytes" => Self::WriteBytes,
            "is_null" => Self::IsNull,
            "from_addr" | "from_addr_mut" | "from_exposed_addr" | "from_exposed_addr_mut" => {
                Self::FromAddress
            }
            "with_exposed_addr" | "with_exposed_addr_mut" => Self::WithAddress,
            "addr" | "expose_addr" => Self::Address,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerConstructor {
    Null,
    Dangling,
}

impl PointerConstructor {
    pub fn resolve(name: &str) -> Option<Self> {
        match name {
            "null" | "null_mut" => Some(Self::Null),
            "dangling" | "dangling_mut" => Some(Self::Dangling),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeapAllocation {
    Uninitialized,
    Zeroed,
}
