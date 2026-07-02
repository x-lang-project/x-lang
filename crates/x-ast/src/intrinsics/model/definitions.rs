use super::*;

impl IntrinsicDefinition for Intrinsic {
    fn resolve(name: &str, scope: IntrinsicScope) -> Option<Self> {
        Self::resolve(name, scope)
    }

    fn signature(self) -> IntrinsicSignature {
        self.signature()
    }

    fn visibility(self) -> IntrinsicVisibility {
        self.visibility()
    }
}

impl IntrinsicDefinition for FloatIntrinsic {
    fn resolve(name: &str, scope: IntrinsicScope) -> Option<Self> {
        Self::parse(name).filter(|value| value.visibility().allows(scope))
    }

    fn signature(self) -> IntrinsicSignature {
        IntrinsicSignature {
            parameters: vec![self.value_type(); self.arity()],
            result: self.result_type(),
        }
    }

    fn visibility(self) -> IntrinsicVisibility {
        if self.width == FloatWidth::F64
            || matches!(
                self.operation,
                FloatOperation::Sqrt
                    | FloatOperation::Floor
                    | FloatOperation::Ceil
                    | FloatOperation::Round
                    | FloatOperation::IsNan
                    | FloatOperation::IsInfinite
            )
        {
            IntrinsicVisibility::SourceAndBackend
        } else {
            IntrinsicVisibility::Unavailable
        }
    }
}

impl IntrinsicDefinition for OverflowIntrinsic {
    fn resolve(name: &str, _scope: IntrinsicScope) -> Option<Self> {
        Self::resolve(name)
    }

    fn signature(self) -> IntrinsicSignature {
        let value = self.value_type();

        IntrinsicSignature {
            parameters: vec![value.clone(); 2],
            result: Type::Tuple(vec![value, Type::Boolean]),
        }
    }

    fn visibility(self) -> IntrinsicVisibility {
        IntrinsicVisibility::SourceAndBackend
    }
}

impl IntrinsicDefinition for RotateDirection {
    fn resolve(name: &str, _scope: IntrinsicScope) -> Option<Self> {
        Self::resolve(name)
    }

    fn signature(self) -> IntrinsicSignature {
        IntrinsicSignature {
            parameters: vec![Type::U32; 2],
            result: Type::U32,
        }
    }

    fn visibility(self) -> IntrinsicVisibility {
        IntrinsicVisibility::SourceAndBackend
    }
}

impl IntrinsicDefinition for BitIntrinsic {
    fn resolve(name: &str, scope: IntrinsicScope) -> Option<Self> {
        Self::parse(name).filter(|value| value.visibility().allows(scope))
    }

    fn signature(self) -> IntrinsicSignature {
        IntrinsicSignature {
            parameters: vec![self.value_type()],
            result: self.value_type(),
        }
    }

    fn visibility(self) -> IntrinsicVisibility {
        if self.operation == BitOperation::ByteSwap || self.width == BitWidth::W32 {
            IntrinsicVisibility::SourceAndBackend
        } else {
            IntrinsicVisibility::BackendOnly
        }
    }
}

impl IntrinsicDefinition for ControlIntrinsic {
    fn resolve(name: &str, _scope: IntrinsicScope) -> Option<Self> {
        match name {
            "trap" | "abort" | "unreachable" | "process_abort" => Some(Self::Abort),
            "process_exit" => Some(Self::Exit),
            _ => None,
        }
    }

    fn signature(self) -> IntrinsicSignature {
        IntrinsicSignature {
            parameters: match self {
                Self::Abort => vec![],
                Self::Exit => vec![Type::I32],
            },
            result: Type::Never,
        }
    }

    fn visibility(self) -> IntrinsicVisibility {
        IntrinsicVisibility::SourceAndBackend
    }
}

impl IntrinsicDefinition for MemoryIntrinsic {
    fn resolve(name: &str, _scope: IntrinsicScope) -> Option<Self> {
        match name {
            "memcpy" => Some(Self::Copy),
            "memmove" => Some(Self::Move),
            "memset" => Some(Self::Set),
            _ => None,
        }
    }

    fn signature(self) -> IntrinsicSignature {
        let pointer = |is_mut| Type::RawPointer {
            is_mut,
            inner: Box::new(Type::Unknown),
        };

        IntrinsicSignature {
            parameters: vec![
                pointer(true),
                match self {
                    Self::Copy | Self::Move => pointer(false),
                    Self::Set => Type::U8,
                },
                Type::USize,
            ],
            result: Type::Void,
        }
    }

    fn visibility(self) -> IntrinsicVisibility {
        IntrinsicVisibility::SourceAndBackend
    }
}

impl IntrinsicDefinition for LayoutIntrinsic {
    fn resolve(name: &str, _scope: IntrinsicScope) -> Option<Self> {
        match name {
            "size_of" => Some(Self::SizeOf),
            "align_of" => Some(Self::AlignOf),
            "offset_of" => Some(Self::OffsetOf),
            _ => None,
        }
    }

    fn signature(self) -> IntrinsicSignature {
        IntrinsicSignature {
            parameters: match self {
                Self::OffsetOf => vec![Type::Unknown],
                _ => vec![],
            },
            result: Type::USize,
        }
    }

    fn visibility(self) -> IntrinsicVisibility {
        IntrinsicVisibility::SourceAndBackend
    }
}
