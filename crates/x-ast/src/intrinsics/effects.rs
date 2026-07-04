use super::{BuiltinCall, BuiltinKind, ControlIntrinsic, Intrinsic, strip_explicit_type_arguments};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectCategory {
    Allocation,
    Random,
    Time,
    Logging,
    Filesystem,
    Network,
    Database,
    Panic,
    Unsafe,
    Atomic,
    Parallel,
    Process,
}

impl EffectCategory {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Allocation => "alloc",
            Self::Random => "random",
            Self::Time => "time",
            Self::Logging => "log",
            Self::Filesystem => "fs",
            Self::Network => "net",
            Self::Database => "db",
            Self::Panic => "panic",
            Self::Unsafe => "unsafe",
            Self::Atomic => "atomic",
            Self::Parallel => "parallel",
            Self::Process => "process",
        }
    }
}

pub fn impure_builtin_category(name: &str) -> Option<&'static str> {
    impure_builtin_effect(name).map(EffectCategory::label)
}

pub fn impure_builtin_effect(name: &str) -> Option<EffectCategory> {
    let call = BuiltinCall::resolve(name, &[], None)?;

    match call.kind {
        BuiltinKind::HeapAllocate(_) => Some(EffectCategory::Allocation),
        BuiltinKind::Print | BuiltinKind::Println => Some(EffectCategory::Logging),
        BuiltinKind::Intrinsic(Intrinsic::Control(ControlIntrinsic::Abort)) => {
            Some(EffectCategory::Panic)
        }
        BuiltinKind::Intrinsic(Intrinsic::Control(ControlIntrinsic::Exit)) => {
            Some(EffectCategory::Process)
        }
        BuiltinKind::Pointer(_)
        | BuiltinKind::CoreMemory(_)
        | BuiltinKind::MemSwap
        | BuiltinKind::MemReplace
        | BuiltinKind::Intrinsic(Intrinsic::Memory(_)) => Some(EffectCategory::Unsafe),
        _ => None,
    }
}

pub fn early_impure_call_category(name: &str) -> Option<&'static str> {
    early_impure_call_effect(name).map(EffectCategory::label)
}

pub fn early_impure_call_effect(name: &str) -> Option<EffectCategory> {
    match strip_explicit_type_arguments(name) {
        "atomic::fence" | "atomic::compiler_fence" => Some(EffectCategory::Atomic),
        _ => impure_builtin_effect(name),
    }
}
