use crate::{Diagnostic, cli};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    Source,
    Internal,
}
impl FailureKind {
    pub(super) fn from_code(code: &str) -> Self {
        if code.starts_with("ICE-") {
            Self::Internal
        } else {
            Self::Source
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StageDiagnostic<'a> {
    pub stage: &'a str,
    pub code: &'a str,
}

impl StageDiagnostic<'static> {
    pub const LLVM_INITIALIZATION: Self = Self::new("LLVM initialisation", "C0017");
    pub const LLVM_TARGET_INITIALIZATION: Self = Self::new("LLVM target initialisation", "C0017");
    pub const LLVM_TARGET_MACHINE_CREATION: Self =
        Self::new("LLVM target machine creation", "C0017");
    pub const LLVM_OBJECT_EMISSION: Self = Self::new("LLVM object emission", "C0018");
    pub const HIR_LOWERING: Self = Self::new("HIR lowering", "H0001");
}

impl<'a> StageDiagnostic<'a> {
    pub const fn new(stage: &'a str, code: &'a str) -> Self {
        Self { stage, code }
    }
}
