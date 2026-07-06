use x_ast::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntegerSuffix {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    I128,
    U128,
    ISize,
    USize,
}

impl IntegerSuffix {
    pub(crate) const ALL: [Self; 12] = [
        Self::ISize,
        Self::USize,
        Self::I128,
        Self::U128,
        Self::I64,
        Self::U64,
        Self::I32,
        Self::U32,
        Self::I16,
        Self::U16,
        Self::I8,
        Self::U8,
    ];

    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::I8 => "i8",
            Self::U8 => "u8",
            Self::I16 => "i16",
            Self::U16 => "u16",
            Self::I32 => "i32",
            Self::U32 => "u32",
            Self::I64 => "i64",
            Self::U64 => "u64",
            Self::I128 => "i128",
            Self::U128 => "u128",
            Self::ISize => "isize",
            Self::USize => "usize",
        }
    }

    pub(crate) fn split(literal: &str) -> (&str, Option<Self>) {
        Self::ALL
            .into_iter()
            .find_map(|suffix| {
                literal
                    .strip_suffix(suffix.spelling())
                    .map(|digits| (digits, Some(suffix)))
            })
            .unwrap_or((literal, None))
    }
}
