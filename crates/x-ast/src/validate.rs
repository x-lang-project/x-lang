use crate::{Expression, Program, Statement};

mod expressions;
mod statements;

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

    fn validate(mut self) -> Result<(), AstValidationError> {
        let mut visited: usize = 0;

        // Push lists in reverse to report the first source error.
        while let Some(node) = self.nodes.pop() {
            visited = visited.saturating_add(1);
            if visited > MAX_AST_NODES {
                return Err(AstValidationError::NodeCount {
                    path: node.path().to_owned(),
                    maximum: MAX_AST_NODES,
                });
            }

            node.validate_depth()?;

            match node {
                Node::Statement(statement, depth, path) => {
                    statements::validate_children(statement, depth, &path, &mut self)?;
                }
                Node::Expression(expression, depth, path) => {
                    expressions::validate_children(expression, depth, &path, &mut self)?;
                }
            }
        }

        Ok(())
    }

    fn push_statements(&mut self, statements: &'a [Statement], depth: usize, path: &str) {
        let detailed = self.detailed;

        for (index, statement) in statements.iter().enumerate().rev() {
            self.push(Node::Statement(
                statement,
                depth,
                diagnostic_path(detailed, || format!("{path}.statement[{index}]")),
            ));
        }
    }

    fn push_expressions(&mut self, expressions: &'a [Expression], depth: usize, path: &str) {
        let detailed = self.detailed;

        for (index, expression) in expressions.iter().enumerate().rev() {
            self.push(Node::Expression(
                expression,
                depth,
                diagnostic_path(detailed, || format!("{path}.expression[{index}]")),
            ));
        }
    }
}

impl Program {
    pub fn validate_ast(&self) -> Result<(), AstValidationError> {
        // Build diagnostic paths only after the fast validation pass fails.
        self.validate_ast_with_paths(false)
            .or_else(|_| self.validate_ast_with_paths(true))
    }

    fn validate_ast_with_paths(&self, detailed: bool) -> Result<(), AstValidationError> {
        let mut work = ValidationWork {
            detailed,
            nodes: Vec::new(),
        };
        work.push_statements(&self.statements, 1, "program");

        work.validate()
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
