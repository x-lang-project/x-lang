use crate::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitWidth {
    W16,
    W32,
    W64,
}

impl BitWidth {
    pub const fn bits(self) -> u32 {
        match self {
            Self::W16 => 16,
            Self::W32 => 32,
            Self::W64 => 64,
        }
    }

    pub fn value_type(self) -> Type {
        match self {
            Self::W16 => Type::U16,
            Self::W32 => Type::U32,
            Self::W64 => Type::U64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowType {
    I32,
    USize,
}

impl OverflowType {
    pub fn value_type(self) -> Type {
        match self {
            Self::I32 => Type::I32,
            Self::USize => Type::USize,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowOperation {
    Add,
    Subtract,
    Multiply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverflowIntrinsic {
    pub operation: OverflowOperation,
    pub value: OverflowType,
}

impl OverflowIntrinsic {
    pub fn value_type(self) -> Type {
        self.value.value_type()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotateDirection {
    Left,
    Right,
}

impl RotateDirection {
    pub fn resolve(name: &str) -> Option<Self> {
        match name {
            "rotl_u32" => Some(Self::Left),
            "rotr_u32" => Some(Self::Right),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitOperation {
    ByteSwap,
    CountOnes,
    LeadingZeros,
    TrailingZeros,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitIntrinsic {
    pub operation: BitOperation,
    pub width: BitWidth,
}

impl BitIntrinsic {
    pub(crate) fn parse(name: &str) -> Option<Self> {
        let (operation, width) = name.rsplit_once('_')?;
        let width = match width {
            "u16" => BitWidth::W16,
            "u32" => BitWidth::W32,
            "u64" => BitWidth::W64,
            _ => return None,
        };

        let operation = match operation {
            "bswap" => BitOperation::ByteSwap,
            "ctpop" => BitOperation::CountOnes,
            "ctlz" => BitOperation::LeadingZeros,
            "cttz" => BitOperation::TrailingZeros,
            _ => return None,
        };

        Some(Self { operation, width })
    }

    pub fn value_type(self) -> Type {
        self.width.value_type()
    }
}
