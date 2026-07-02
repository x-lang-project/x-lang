use crate::{
    BitIntrinsic, BitOperation, BitWidth, FloatIntrinsic, FloatOperation, FloatWidth,
    OverflowIntrinsic, RotateDirection, Type,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicScope {
    Source,
    Backend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicVisibility {
    SourceAndBackend,
    BackendOnly,
    Unavailable,
}

impl IntrinsicVisibility {
    pub const fn allows(self, scope: IntrinsicScope) -> bool {
        matches!(
            (self, scope),
            (Self::SourceAndBackend, _) | (Self::BackendOnly, IntrinsicScope::Backend)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntrinsicSignature {
    pub parameters: Vec<Type>,
    pub result: Type,
}

pub trait IntrinsicDefinition: Sized + Copy {
    fn resolve(name: &str, scope: IntrinsicScope) -> Option<Self>;
    fn signature(self) -> IntrinsicSignature;
    fn visibility(self) -> IntrinsicVisibility;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlIntrinsic {
    Abort,
    Exit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryIntrinsic {
    Copy,
    Move,
    Set,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutIntrinsic {
    SizeOf,
    AlignOf,
    OffsetOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intrinsic {
    Float(FloatIntrinsic),
    Overflow(OverflowIntrinsic),
    Rotate(RotateDirection),
    Bit(BitIntrinsic),
    Control(ControlIntrinsic),
    Memory(MemoryIntrinsic),
    Layout(LayoutIntrinsic),
}

impl Intrinsic {
    pub fn resolve(name: &str, scope: IntrinsicScope) -> Option<Self> {
        <FloatIntrinsic as IntrinsicDefinition>::resolve(name, scope)
            .map(Self::Float)
            .or_else(|| {
                <OverflowIntrinsic as IntrinsicDefinition>::resolve(name, scope).map(Self::Overflow)
            })
            .or_else(|| {
                <RotateDirection as IntrinsicDefinition>::resolve(name, scope).map(Self::Rotate)
            })
            .or_else(|| <BitIntrinsic as IntrinsicDefinition>::resolve(name, scope).map(Self::Bit))
            .or_else(|| ControlIntrinsic::resolve(name, scope).map(Self::Control))
            .or_else(|| MemoryIntrinsic::resolve(name, scope).map(Self::Memory))
            .or_else(|| LayoutIntrinsic::resolve(name, scope).map(Self::Layout))
            .filter(|intrinsic| intrinsic.visibility().allows(scope))
    }

    pub fn signature(self) -> IntrinsicSignature {
        match self {
            Self::Float(value) => value.signature(),
            Self::Overflow(value) => value.signature(),
            Self::Rotate(value) => value.signature(),
            Self::Bit(value) => value.signature(),
            Self::Control(value) => value.signature(),
            Self::Memory(value) => value.signature(),
            Self::Layout(value) => value.signature(),
        }
    }

    pub fn visibility(self) -> IntrinsicVisibility {
        match self {
            Self::Float(value) => value.visibility(),
            Self::Overflow(value) => value.visibility(),
            Self::Rotate(value) => value.visibility(),
            Self::Bit(value) => value.visibility(),
            Self::Control(value) => value.visibility(),
            Self::Memory(value) => value.visibility(),
            Self::Layout(value) => value.visibility(),
        }
    }
}

mod definitions;
