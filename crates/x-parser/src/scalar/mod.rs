use crate::ParsedModule;

use x_ast::{Program, SourceSpan, Type};

mod lexer;
mod literals;

type Attempt<T> = Option<T>;

fn name_allowed(word: &str) -> bool {
    let first = word.as_bytes().first();

    first.is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
        && !word.starts_with("break")
        && !word.starts_with("continue")
        && !matches!(
            word,
            "pure"
                | "memoise"
                | "parallel"
                | "static"
                | "unique"
                | "throws"
                | "invariant"
                | "layout"
                | "multi"
                | "become"
                | "fn"
                | "let"
                | "const"
                | "struct"
                | "return"
                | "if"
                | "else"
                | "true"
                | "false"
                | "import"
                | "mod"
                | "throw"
                | "extern"
                | "while"
                | "for"
                | "in"
                | "trait"
                | "impl"
                | "pub"
                | "mut"
                | "as"
                | "sizeof"
                | "unsafe"
                | "match"
                | "do"
                | "type"
        )
}
