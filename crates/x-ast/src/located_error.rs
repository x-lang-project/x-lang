use std::fmt;

use crate::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedError<E> {
    pub span: SourceSpan,
    pub error: Box<E>,
}

impl<E: fmt::Display> fmt::Display for LocatedError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(formatter)
    }
}

pub trait SourceError: Sized {
    fn location(&self) -> Option<&LocatedError<Self>>;
    fn with_location(self, location: SourceSpan) -> Self;

    fn source_span(&self) -> Option<SourceSpan> {
        self.location().map(|location| location.span)
    }

    fn at_source(self, span: Option<SourceSpan>) -> Self {
        if self.location().is_some() {
            return self;
        }

        match span.filter(|span| span.file_identifier.is_some()) {
            Some(span) => self.with_location(span),
            None => self,
        }
    }
}

#[macro_export]
macro_rules! impl_source_error {
    ($error:ty) => {
        impl $crate::SourceError for $error {
            fn location(&self) -> Option<&$crate::LocatedError<Self>> {
                match self {
                    Self::Located(location) => Some(location),
                    _ => None,
                }
            }
            fn with_location(self, span: $crate::SourceSpan) -> Self {
                Self::Located($crate::LocatedError {
                    span,
                    error: Box::new(self),
                })
            }
        }
    };
}
