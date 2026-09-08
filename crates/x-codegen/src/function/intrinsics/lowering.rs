use std::borrow::Cow;

use x_ast::{BitIntrinsic, BitOperation, BitWidth, FloatIntrinsic, FloatOperation, FloatWidth};

// LLVM's is_zero_undef must be false for X's defined zero result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ZeroInput {
    NoFlag,
    Defined,
}

pub(super) struct LlvmDefinition {
    pub symbol: Option<Cow<'static, str>>,
    pub zero_input: ZeroInput,
}

pub(super) trait LlvmIntrinsicDefinition {
    fn llvm_definition(self) -> LlvmDefinition;
}

impl LlvmIntrinsicDefinition for BitIntrinsic {
    fn llvm_definition(self) -> LlvmDefinition {
        let operation = match self.operation {
            BitOperation::ByteSwap => "bswap",
            BitOperation::CountOnes => "ctpop",
            BitOperation::LeadingZeros => "ctlz",
            BitOperation::TrailingZeros => "cttz",
        };
        let width = match self.width {
            BitWidth::W16 => 16,
            BitWidth::W32 => 32,
            BitWidth::W64 => 64,
        };

        LlvmDefinition {
            symbol: Some(format!("llvm.{operation}.i{width}").into()),
            zero_input: match self.operation {
                BitOperation::LeadingZeros | BitOperation::TrailingZeros => ZeroInput::Defined,
                BitOperation::ByteSwap | BitOperation::CountOnes => ZeroInput::NoFlag,
            },
        }
    }
}

impl LlvmIntrinsicDefinition for FloatIntrinsic {
    fn llvm_definition(self) -> LlvmDefinition {
        let symbol = match (self.operation, self.width) {
            (FloatOperation::Sqrt, FloatWidth::F32) => Some("sqrtf"),
            (FloatOperation::Floor, FloatWidth::F32) => Some("floorf"),
            (FloatOperation::Ceil, FloatWidth::F32) => Some("ceilf"),
            (FloatOperation::Round, FloatWidth::F32) => Some("roundf"),
            (FloatOperation::Sqrt, FloatWidth::F64) => Some("sqrt"),
            (FloatOperation::Floor, FloatWidth::F64) => Some("floor"),
            (FloatOperation::Ceil, FloatWidth::F64) => Some("ceil"),
            (FloatOperation::Round, FloatWidth::F64) => Some("round"),
            (FloatOperation::Sin, FloatWidth::F64) => Some("sin"),
            (FloatOperation::Cos, FloatWidth::F64) => Some("cos"),
            (FloatOperation::Tan, FloatWidth::F64) => Some("tan"),
            (FloatOperation::Asin, FloatWidth::F64) => Some("asin"),
            (FloatOperation::Acos, FloatWidth::F64) => Some("acos"),
            (FloatOperation::Atan, FloatWidth::F64) => Some("atan"),
            (FloatOperation::Atan2, FloatWidth::F64) => Some("atan2"),
            (FloatOperation::Exp, FloatWidth::F64) => Some("exp"),
            (FloatOperation::Ln, FloatWidth::F64) => Some("log"),
            (FloatOperation::Pow, FloatWidth::F64) => Some("pow"),
            _ => None,
        };

        LlvmDefinition {
            symbol: symbol.map(Into::into),
            zero_input: ZeroInput::NoFlag,
        }
    }
}
