use crate::Type;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DeclarationKey {
    Function(String, Vec<Type>),
    Extern(String),
    Struct(String),
    Trait(String),
    Impl(String, Type),
    Alias(String),
    Constant(String),
    Binding(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    pub file_identifier: Option<u32>,
    pub start: usize,
    pub end: usize,
}

impl SourceSpan {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            file_identifier: None,
            start,
            end,
        }
    }

    pub fn join(left: Option<Self>, right: Option<Self>) -> Option<Self> {
        match (left, right) {
            (Some(left), Some(right)) => Some(Self {
                file_identifier: left.file_identifier.or(right.file_identifier),
                start: left.start.min(right.start),
                end: left.end.max(right.end),
            }),
            (Some(span), None) | (None, Some(span)) => Some(span),
            (None, None) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Visibility {
    Private,
    Public,
    PublicCrate,
    PublicIn(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Attribute {
    pub path: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Lifetime {
    Static,
    Named(String),
    Infer(usize),
    Erased,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericParameter {
    Type {
        name: String,
        bounds: Vec<Type>,
        default: Option<Type>,
    },
    Lifetime {
        name: String,
        bounds: Vec<Lifetime>,
    },
    Const {
        name: String,
        value_type: Type,
        default: Option<ConstValue>,
    },
}

impl GenericParameter {
    pub fn non_lifetime_name(&self) -> Option<&String> {
        match self {
            Self::Type { name, .. } | Self::Const { name, .. } => Some(name),
            Self::Lifetime { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericArgument {
    Type(Type),
    Lifetime(Lifetime),
    Const(ConstValue),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstValue {
    Integer(i64),
    UInteger(u64),
    Boolean(bool),
    Character(char),
    String(String),
    Type(Type),
}
