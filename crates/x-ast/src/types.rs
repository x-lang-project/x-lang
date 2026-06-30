use crate::Lifetime;

mod children;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferenceQualifiers {
    pub is_mut: bool,
    pub is_unique: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    ShiftLeft,
    ShiftRight,
    BitAnd,
    Xor,
    BitOr,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
    Or,
    And,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Default)]
pub enum Layout {
    #[default]
    AoS,
    SoA,
    Auto,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate,
    LogicalNot,
    BitwiseNot,
    PreIncrement,
    PreDecrement,
    PostIncrement,
    PostDecrement,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    ISize,
    USize,
    F32,
    F64,
    String,
    Character,
    Boolean,
    Void,
    Never,
    Unknown,
    Infer(usize),
    Custom(String),
    Array(Box<Type>),
    FixedSizedArray(Box<Type>, usize),
    Tuple(Vec<Type>),
    Simd {
        element: Box<Type>,
        lanes: usize,
    },

    RawPointer {
        is_mut: bool,
        inner: Box<Type>,
    },

    Reference {
        lifetime: Option<Lifetime>,
        is_mut: bool,
        is_unique: bool,
        inner: Box<Type>,
    },

    TypeParameter(String),
    GenericInstance {
        name: String,
        type_arguments: Vec<Type>,
    },
    Function {
        parameters: Vec<Type>,
        return_value: Box<Type>,
    },
}

impl Type {
    pub fn pointee(&self) -> Option<&Self> {
        match self {
            Self::RawPointer { inner, .. } | Self::Reference { inner, .. } => Some(inner.as_ref()),
            _ => None,
        }
    }

    pub fn any_nested_type(&self, predicate: impl Fn(&Self) -> bool + Copy) -> bool {
        if predicate(self) {
            return true;
        }

        self.children()
            .any(|inner| inner.any_nested_type(predicate))
    }

    pub fn is_scalar(&self) -> bool {
        self.is_integer() || matches!(self, Self::F32 | Self::F64 | Self::Boolean)
    }

    pub fn from_primitive_name(name: &str) -> Option<Self> {
        Some(match name {
            "i8" => Self::I8,
            "u8" => Self::U8,
            "i16" => Self::I16,
            "u16" => Self::U16,
            "i32" => Self::I32,
            "u32" => Self::U32,
            "i64" => Self::I64,
            "u64" => Self::U64,
            "isize" => Self::ISize,
            "usize" => Self::USize,
            "f32" => Self::F32,
            "f64" => Self::F64,
            "str" => Self::String,
            "char" => Self::Character,
            "bool" => Self::Boolean,
            "void" => Self::Void,
            _ => return None,
        })
    }

    pub fn from_external_name(name: &str) -> Self {
        let name = name.trim();

        for (prefix, is_mut) in [("*mut ", true), ("*const ", false)] {
            if let Some(inner) = name.strip_prefix(prefix) {
                return Self::RawPointer {
                    is_mut,
                    inner: Box::new(Self::from_external_name(inner)),
                };
            }
        }

        Self::from_primitive_name(name).unwrap_or_else(|| Self::Custom(name.to_string()))
    }

    // Char uses integer storage during coercion and lowering.
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Self::I8
                | Self::U8
                | Self::I16
                | Self::U16
                | Self::I32
                | Self::U32
                | Self::I64
                | Self::U64
                | Self::ISize
                | Self::USize
                | Self::Character
        )
    }
}
