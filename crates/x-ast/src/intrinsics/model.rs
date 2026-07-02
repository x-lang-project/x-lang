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
