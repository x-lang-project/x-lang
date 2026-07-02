use std::fmt;

use crate::{ExpressionKind, Lifetime, StringPart, Type};

impl fmt::Display for StringPart {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StringPart::Text(text) => write!(formatter, "{text}"),
            StringPart::Interpolation(expression) => match &expression.kind {
                ExpressionKind::String(str_literal) => {
                    for part in &str_literal.parts {
                        write!(formatter, "{part}")?;
                    }

                    Ok(())
                }
                ExpressionKind::Identifier(name) => write!(formatter, "{name}"),
                _ => write!(formatter, "{expression:?}"),
            },
        }
    }
}

struct DisplayTypes<'a>(&'a [Type]);

impl fmt::Display for Lifetime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Lifetime::Static => write!(formatter, "'static"),
            Lifetime::Named(name) => write!(formatter, "{name}"),
            Lifetime::Infer(identifier) => write!(formatter, "'_{identifier}"),
            Lifetime::Erased => write!(formatter, "'_"),
        }
    }
}
