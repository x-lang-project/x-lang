use super::BuiltinKind;
use super::ControlIntrinsic;
use super::Intrinsic;
use super::strip_explicit_type_arguments;

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
