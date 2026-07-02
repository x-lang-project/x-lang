use crate::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatWidth {
    F32,
    F64,
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
    pub(crate) fn parse(name: &str) -> Option<Self> {
        let (width, operation) = name.split_once('_')?;
        let width = match width {
            _ => return None,
        };

        let operation = match operation {
            _ => return None,
        };

        Some(Self { operation, width })
    }

    pub fn arity(self) -> usize {
        if matches!(self.operation, FloatOperation::Atan2 | FloatOperation::Pow) {
            2
        } else {
            1
        }
    }
}
