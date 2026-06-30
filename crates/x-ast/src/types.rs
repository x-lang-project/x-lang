use crate::Lifetime;

mod children;

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
