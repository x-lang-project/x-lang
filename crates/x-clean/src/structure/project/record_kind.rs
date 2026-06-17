use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RecordKind {
    Module,
    Cfg,
    Record,
    Derive,
    Enum,
    Macro,
    Impl,
    ErrorMap,
    Use,
    Reexport,
    ImportAlias,
}
impl fmt::Display for RecordKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Module => "module",
            Self::Cfg => "cfg",
            Self::Record => "record",
            Self::Derive => "derive",
            Self::Enum => "enum",
            Self::Macro => "macro",
            Self::Impl => "impl",
            Self::ErrorMap => "error-map",
            Self::Use => "use",
            Self::Reexport => "reexport",
            Self::ImportAlias => "import-alias",
        })
    }
}
#[derive(Clone, Copy)]
pub(super) enum AttributePolicy {
    Cfg,
    Derive,
}
impl AttributePolicy {
    pub(super) fn syntax(self) -> &'static str {
        match self {
            Self::Cfg => "cfg",
            Self::Derive => "derive",
        }
    }

    pub(super) fn kind(self) -> RecordKind {
        match self {
            Self::Cfg => RecordKind::Cfg,
            Self::Derive => RecordKind::Derive,
        }
    }
}
