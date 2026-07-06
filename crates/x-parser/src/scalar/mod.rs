use crate::ParsedModule;

use x_ast::{Program, SourceSpan, Type};

mod lexer;

type Attempt<T> = Option<T>;
