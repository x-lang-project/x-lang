use std::iter::Peekable;

use pest::iterators::Pairs;

use crate::Rule;
use x_ast::{Lifetime, Type};

#[derive(Default)]
pub(crate) struct ReferenceSyntax {
    lifetime: Option<Lifetime>,
    pub(crate) is_mut: bool,
    is_unique: bool,
}

impl ReferenceSyntax {
    pub(crate) fn consume(inner: &mut Peekable<Pairs<'_, Rule>>) -> Self {
        let mut syntax = Self::default();

        while matches!(
            inner.peek().map(|pair| pair.as_rule()),
            Some(Rule::kw_static | Rule::unique_kw | Rule::kw_mut | Rule::lifetime)
        ) {
            let Some(pair) = inner.next() else {
                break;
            };

            match pair.as_rule() {
                Rule::kw_mut => syntax.is_mut = true,
                Rule::unique_kw => syntax.is_unique = true,
                Rule::kw_static => syntax.lifetime = Some(Lifetime::Static),
                Rule::lifetime => {
                    syntax.lifetime = Some(match pair.as_str() {
                        "'static" => Lifetime::Static,
                        name => Lifetime::Named(name.to_string()),
                    });
                }
                _ => {}
            }
        }

        syntax
    }

    pub(crate) fn into_type(self, inner: Type) -> Type {
        Type::Reference {
            lifetime: self.lifetime,
            is_mut: self.is_mut,
            is_unique: self.is_unique,
            inner: Box::new(inner),
        }
    }
}
