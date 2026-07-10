mod declarations;

use super::{Attempt, Scalar};
use x_ast::{Expression, ExpressionKind, Parameter, Pattern, Statement, Type};

impl Scalar<'_> {
    pub(super) fn statement(&mut self, in_block: bool) -> Attempt<Statement> {
        let statement = self.statement_inner(in_block)?;

        if !matches!(statement, Statement::If { .. }) {
            self.statement_end = self.tokens.get(self.position.checked_sub(1)?)?.end;
        }

        Some(statement)
    }

    fn statement_inner(&mut self, in_block: bool) -> Attempt<Statement> {
        let token = self.peek();

        match token {
            "fn" | "pure" | "memoise" | "static" | "multi" | "throws" | "unsafe" | "pub" => {
                self.function()
            }
            "{" => Some(Statement::Block {
                statements: self.block()?,
            }),
            "let" | "const" => {
                let constant = self.next().text == "const";

                if !constant {
                    while self.eat("mut") || self.eat("unique") {}
                }

                let name = self.name()?;
                let type_annotation = if self.eat(":") {
                    Some(self.value_type()?)
                } else {
                    None
                };

                self.need("=")?;
                let value = self.expression(0)?;
                self.need(";")?;

                Some(if constant {
                    Statement::Constant {
                        name,
                        type_annotation,
                        value,
                    }
                } else {
                    Statement::VariableDeclaration {
                        pattern: Pattern::Identifier(name),
                        type_annotation,
                        value,
                    }
                })
            }
            "return" => {
                self.next();
                let value = if self.peek() == ";" {
                    None
                } else {
                    Some(self.expression(0)?)
                };

                self.need(";")?;

                Some(Statement::Return { value })
            }
            "if" => self.if_statement(),
            "while" => {
                self.next();
                let condition = Box::new(self.expression(0)?);
                let body = self.block()?;

                Some(Statement::WhileLoop { condition, body })
            }
            "loop" => {
                self.next();

                Some(Statement::WhileLoop {
                    condition: Box::new(Expression {
                        span: None,
                        value_type: Type::Boolean,
                        kind: ExpressionKind::Boolean(true),
                    }),
                    body: self.block()?,
                })
            }
            "break" | "continue" => {
                let word = self.next().text;
                self.need(";")?;

                Some(if word == "break" {
                    Statement::Break
                } else {
                    Statement::Continue
                })
            }
            "import" | "use" => self.import(),
            "mod" => {
                self.next();
                let name = self.name()?;
                self.need(";")?;
                if !in_block {
                    self.submodules.push(name.clone());
                }

                Some(Statement::Comment(format!("mod {name}")))
            }
            "type" => {
                self.next();
                let name = self.name()?;
                self.need("=")?;
                let value_type = self.value_type()?;
                self.need(";")?;

                Some(Statement::TypeAlias {
                    name,
                    generic_parameters: Vec::new(),
                    value_type,
                })
            }
            _ => {
                // Match Pest's joined statement-keyword prefixes.
                if [
                    "fn",
                    "let",
                    "const",
                    "return",
                    "if",
                    "while",
                    "loop",
                    "throw",
                    "panic",
                    "do",
                    "become",
                    "mod",
                    "type",
                    "invariant",
                    "unsafe",
                    "import",
                    "use",
                    "pub",
                    "struct",
                    "trait",
                    "impl",
                    "extern",
                    "for",
                ]
                .iter()
                .any(|word| token.starts_with(word))
                {
                    return None;
                }

                let inner_expression = self.expression(0)?;

                if self.eat("=") {
                    if !matches!(inner_expression.kind, ExpressionKind::Identifier(_)) {
                        return None;
                    }

                    let value = self.expression(0)?;
                    self.need(";")?;

                    return Some(Statement::Expression {
                        expression: Expression {
                            span: None,
                            value_type: Type::Void,
                            kind: ExpressionKind::Assignment {
                                target: Box::new(inner_expression),
                                value: Box::new(value),
                            },
                        },
                    });
                }

                if self.eat(";") {
                    Some(Statement::Expression {
                        expression: inner_expression,
                    })
                } else if in_block && self.peek() == "}" {
                    Some(Statement::Return {
                        value: Some(inner_expression),
                    })
                } else {
                    None
                }
            }
        }
    }
}
