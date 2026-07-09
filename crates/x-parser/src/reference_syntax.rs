use std::iter::Peekable;

use pest::iterators::Pairs;

use x_ast::{Lifetime, Type};

#[derive(Default)]
pub(crate) struct ReferenceSyntax {
    lifetime: Option<Lifetime>,
    pub(crate) is_mut: bool,
    is_unique: bool,
}

impl ReferenceSyntax {
    pub(crate) fn into_type(self, inner: Type) -> Type {
        Type::Reference {
            lifetime: self.lifetime,
            is_mut: self.is_mut,
            is_unique: self.is_unique,
            inner: Box::new(inner),
        }
    }
}
