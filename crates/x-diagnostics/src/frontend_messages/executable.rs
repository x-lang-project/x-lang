use std::error;
use std::fmt;

use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutableTypeReason {
    SoaWholeValueAddress,
    MissingEnumDiscriminantAbi,
    GenericMethodBody,
    UninstantiatedGenericInvariant,
    UnsizedArrayLength,
    StringInterpolation,
    RecursiveStorageCycle,
}

impl ExecutableTypeReason {
    pub fn display_diagnostic(&self) -> crate::Diagnostic {
        match self {
            Self::StringInterpolation => {
                Error::new("string interpolation requires executable runtime formatting support")
            }
            Self::RecursiveStorageCycle => {
                Error::new("recursive by-value storage cycle has no finite representation")
            }
            Self::SoaWholeValueAddress => Error::new(
                "a layout(SoA) array element has no contiguous whole-value address; project a field such as `items[i].x`",
            ),
            Self::MissingEnumDiscriminantAbi => {
                Error::new("pattern matching requires a discriminant-preserving enum ABI")
            }
            Self::GenericMethodBody => Error::new("method type parameters are unsupported"),
            Self::UninstantiatedGenericInvariant => Error::new(
                "generic struct invariants require a concrete instantiation before boundary instrumentation",
            ),
            Self::UnsizedArrayLength => Error::new(
                "an unsized array carries no runtime length; use a fixed array or a slice descriptor",
            ),
        }
    }
}

impl fmt::Display for ExecutableTypeReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.display_diagnostic().render_detail())
    }
}

impl error::Error for ExecutableTypeReason {}
