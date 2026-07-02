#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignOperation {
    IsPowerOfTwo,
    IsAligned,
    AlignUp,
    PageAlignUp,
    AlignPointer,
    AlignDown,
    PageAlignDown,
    PaddingNeeded,
    Maximum,
    Minimum,
}

impl AlignOperation {
    pub fn resolve(leaf: &str) -> Option<Self> {
        Some(match leaf {
            "is_power_of_two" => Self::IsPowerOfTwo,
            "is_aligned" => Self::IsAligned,
            "align_up" => Self::AlignUp,
            "page_align_up" => Self::PageAlignUp,
            "align_ptr" => Self::AlignPointer,
            "align_down" => Self::AlignDown,
            "page_align_down" => Self::PageAlignDown,
            "padding_needed" | "padding_needed_for" => Self::PaddingNeeded,
            "max_align" => Self::Maximum,
            "min_align" => Self::Minimum,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberOperation {
    TestSignExtension,
    TestTruncation,
    SaturatingSubtractUsize,
    CheckedMultiplyUsize,
    CheckedAddUsize,
    CheckedSubtractUsize,
    AbsoluteI32,
    Minimum,
    Maximum,
    Clamp,
    SignExtendU32,
    GetBitsU32,
    SetBitsU32,
    CountOnesU32,
    ByteSwapU16,
}

impl NumberOperation {
    pub fn resolve(leaf: &str) -> Option<Self> {
        Some(match leaf {
            "max" => Self::Maximum,
            "clamp" => Self::Clamp,
            "sign_extend_u32" => Self::SignExtendU32,
            "get_bits_u32" => Self::GetBitsU32,
            "set_bits_u32" => Self::SetBitsU32,
            "count_ones_u32" => Self::CountOnesU32,
            "byte_swap_u16" => Self::ByteSwapU16,
            _ => return None,
        })
    }

    pub const fn source_name(self) -> &'static str {
        match self {
            Self::TestSignExtension => "test_sign_extension",
            Self::TestTruncation => "test_truncation",
            Self::SaturatingSubtractUsize => "saturating_sub_usize",
            Self::CheckedMultiplyUsize => "checked_mul_usize",
            Self::CheckedAddUsize => "checked_add_usize",
            Self::CheckedSubtractUsize => "checked_sub_usize",
            Self::AbsoluteI32 => "abs_i32",
            Self::Minimum => "min",
            Self::Maximum => "max",
            Self::Clamp => "clamp",
            Self::SignExtendU32 => "sign_extend_u32",
            Self::GetBitsU32 => "get_bits_u32",
            Self::SetBitsU32 => "set_bits_u32",
            Self::CountOnesU32 => "count_ones_u32",
            Self::ByteSwapU16 => "byte_swap_u16",
        }
    }
}
