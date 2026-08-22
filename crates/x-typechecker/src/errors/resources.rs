#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeResource {
    ExpressionDepth,
    TypeResolutionDepth,
    GenericStructInstantiationDepth,
    RecordStorageDepth,
    StaticSpecializations,
}

impl TypeResource {
    pub const fn label(self) -> &'static str {
        match self {
            Self::ExpressionDepth => "expression depth",
            Self::TypeResolutionDepth => "type resolution depth",
            Self::GenericStructInstantiationDepth => "generic struct instantiation depth",
            Self::RecordStorageDepth => "record storage nesting depth",
            Self::StaticSpecializations => "static specialisations",
        }
    }
}
