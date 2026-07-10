use crate::ParsedModule;
use lexer::{Token, lex};
use x_ast::{Program, SourceSpan, Type};

mod expressions;
mod lexer;
mod literals;
mod statements;
struct Scalar<'a> {
    input: &'a str,
    tokens: Vec<Token<'a>>,
    position: usize,
    submodules: Vec<String>,
    statement_end: usize,
}
type Attempt<T> = Option<T>;

pub(super) fn try_parse(input: &str) -> Attempt<ParsedModule> {
    let tokens = lex(input)?;
    let mut parser = Scalar {
        input,
        tokens,
        position: 0,
        submodules: Vec::new(),
        statement_end: 0,
    };
    let mut statements = Vec::new();
    let mut item_spans = Vec::new();

    while parser.peek() != "" {
        let start = parser.current().start;
        statements.push(parser.statement(false)?);
        let end = parser.statement_end;
        item_spans.push(Some(SourceSpan::new(start, end)));
    }

    Some(ParsedModule {
        program: Program { statements },
        item_spans,
        submodules: parser.submodules,
    })
}

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

impl<'a> Scalar<'a> {
    fn current(&self) -> Token<'a> {
        self.tokens.get(self.position).copied().unwrap_or(Token {
            text: "",
            start: self.input.len(),
            end: self.input.len(),
        })
    }

    fn peek(&self) -> &'a str {
        self.current().text
    }

    fn next(&mut self) -> Token<'a> {
        let token = self.current();
        self.position += 1;

        token
    }

    fn eat(&mut self, text: &str) -> bool {
        if self.peek() == text {
            self.next();

            true
        } else {
            false
        }
    }

    fn need(&mut self, text: &str) -> Attempt<()> {
        self.eat(text).then_some(())
    }

    fn name(&mut self) -> Attempt<String> {
        name_allowed(self.peek()).then(|| self.next().text.to_owned())
    }

    fn value_type(&mut self) -> Attempt<Type> {
        let token = self.current();
        let text = token.text;

        if matches!(text, "i128" | "u128" | "simd") {
            return None;
        }

        let primitive = Type::from_primitive_name(text);

        if primitive.is_none() && !name_allowed(text) {
            return None;
        }

        self.next();
        let value_type = primitive.unwrap_or_else(|| {
            Type::Custom(
                self.input
                    .get(token.start..self.current().start)
                    .unwrap_or(text)
                    .trim()
                    .to_owned(),
            )
        });

        if matches!(self.peek(), "<" | "::" | "[" | "&") {
            return None;
        }

        Some(value_type)
    }
}
