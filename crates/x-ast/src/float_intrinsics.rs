use crate::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatWidth {
    F32,
    F64,
}

impl FloatWidth {
    pub fn value_type(self) -> Type {
        match self {
            Self::F32 => Type::F32,
            Self::F64 => Type::F64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatOperation {
    Sqrt,
    Floor,
    Ceil,
    Round,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
    Exp,
    Ln,
    Pow,
    IsNan,
    IsInfinite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FloatIntrinsic {
    pub operation: FloatOperation,
    pub width: FloatWidth,
}

impl FloatIntrinsic {
    pub fn resolve(name: &str) -> Option<Self> {
        use crate::intrinsics::{IntrinsicDefinition, IntrinsicScope};

        <Self as IntrinsicDefinition>::resolve(name, IntrinsicScope::Source)
    }

    pub(crate) fn parse(name: &str) -> Option<Self> {
        let (width, operation) = name.split_once('_')?;
        let width = match width {
            "f32" => FloatWidth::F32,
            "f64" => FloatWidth::F64,
            _ => return None,
        };

        let operation = match operation {
            "sqrt" => FloatOperation::Sqrt,
            "floor" => FloatOperation::Floor,
            "ceil" => FloatOperation::Ceil,
            "round" => FloatOperation::Round,
            "is_nan" => FloatOperation::IsNan,
            "is_infinite" => FloatOperation::IsInfinite,
            "sin" => FloatOperation::Sin,
            "cos" => FloatOperation::Cos,
            "tan" => FloatOperation::Tan,
            "asin" => FloatOperation::Asin,
            "acos" => FloatOperation::Acos,
            "atan" => FloatOperation::Atan,
            "atan2" => FloatOperation::Atan2,
            "exp" => FloatOperation::Exp,
            "ln" => FloatOperation::Ln,
            "powf" => FloatOperation::Pow,
            _ => return None,
        };

        Some(Self { operation, width })
    }

    pub fn value_type(self) -> Type {
        self.width.value_type()
    }

    pub fn result_type(self) -> Type {
        if matches!(
            self.operation,
            FloatOperation::IsNan | FloatOperation::IsInfinite
        ) {
            Type::Boolean
        } else {
            self.value_type()
        }
    }

    pub fn arity(self) -> usize {
        if matches!(self.operation, FloatOperation::Atan2 | FloatOperation::Pow) {
            2
        } else {
            1
        }
    }
}
