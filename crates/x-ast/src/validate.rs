use crate::{Expression, Program, Statement};

pub const MAX_AST_EXPRESSION_DEPTH: usize = 16;
pub const MAX_AST_STATEMENT_DEPTH: usize = 32;
pub const MAX_AST_NODES: usize = 250_000;

pub use x_diagnostics::frontend_messages::AstValidationError;

enum Node<'a> {
    Statement(&'a Statement, usize, String),
    Expression(&'a Expression, usize, String),
}

struct ValidationWork<'a> {
    detailed: bool,
    nodes: Vec<Node<'a>>,
}

impl<'a> ValidationWork<'a> {
    fn push(&mut self, node: Node<'a>) {
        self.nodes.push(node);
    }
}

impl Node<'_> {
    fn path(&self) -> &str {
        match self {
            Self::Statement(_, _, path) | Self::Expression(_, _, path) => path,
        }
    }

    fn validate_depth(&self) -> Result<(), AstValidationError> {
        match self {
            Self::Statement(_, depth, path) if *depth > MAX_AST_STATEMENT_DEPTH => {
                Err(AstValidationError::StatementDepth {
                    path: path.clone(),
                    maximum: MAX_AST_STATEMENT_DEPTH,
                })
            }
            Self::Expression(_, depth, path) if *depth > MAX_AST_EXPRESSION_DEPTH => {
                Err(AstValidationError::ExpressionDepth {
                    path: path.clone(),
                    maximum: MAX_AST_EXPRESSION_DEPTH,
                })
            }
            _ => Ok(()),
        }
    }
}

fn diagnostic_path(detailed: bool, build: impl FnOnce() -> String) -> String {
    if detailed { build() } else { String::new() }
}

fn require_name(name: &str, path: String) -> Result<(), AstValidationError> {
    if name.trim().is_empty() {
        Err(AstValidationError::EmptySymbol { path })
    } else {
        Ok(())
    }
}
