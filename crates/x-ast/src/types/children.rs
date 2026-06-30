use super::Type;

impl Type {
    pub(crate) fn children(&self) -> impl Iterator<Item = &Self> {
        let (items, last): (&[Self], Option<&Self>) = match self {
            Self::Array(inner)
            | Self::FixedSizedArray(inner, _)
            | Self::Simd { element: inner, .. }
            | Self::RawPointer { inner, .. }
            | Self::Reference { inner, .. } => (&[], Some(inner)),
            Self::Tuple(items)
            | Self::GenericInstance {
                type_arguments: items,
                ..
            } => (items, None),
            Self::Function {
                parameters,
                return_value,
            } => (parameters, Some(return_value)),
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
            | Self::F32
            | Self::F64
            | Self::String
            | Self::Character
            | Self::Boolean
            | Self::Void
            | Self::Never
            | Self::Unknown
            | Self::Infer(_)
            | Self::Custom(_)
            | Self::TypeParameter(_) => (&[], None),
        };

        items.iter().chain(last)
    }

    pub(crate) fn children_mut(&mut self) -> impl Iterator<Item = &mut Self> {
        let (items, last): (&mut [Self], Option<&mut Self>) = match self {
            Self::Array(inner)
            | Self::FixedSizedArray(inner, _)
            | Self::Simd { element: inner, .. }
            | Self::RawPointer { inner, .. }
            | Self::Reference { inner, .. } => (&mut [], Some(inner)),
            Self::Tuple(items)
            | Self::GenericInstance {
                type_arguments: items,
                ..
            } => (items, None),
            Self::Function {
                parameters,
                return_value,
            } => (parameters, Some(return_value)),
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
            | Self::F32
            | Self::F64
            | Self::String
            | Self::Character
            | Self::Boolean
            | Self::Void
            | Self::Never
            | Self::Unknown
            | Self::Infer(_)
            | Self::Custom(_)
            | Self::TypeParameter(_) => (&mut [], None),
        };

        items.iter_mut().chain(last)
    }
}
