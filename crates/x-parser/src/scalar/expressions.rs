use super::literals::{decode_string, numeric};
use super::{Attempt, Scalar, name_allowed};
use x_ast::{
    Expression, ExpressionKind, Operator, SourceSpan, StringLiteral, StringPart, Type,
    UnaryOperator,
};

impl Scalar<'_> {
    pub(super) fn expression(&mut self, minimum: u8) -> Attempt<Expression> {
        let mut current_left = self.unary()?;

        while let Some((precedence, operator)) = binary(self.peek()) {
            if precedence < minimum {
                break;
            }

            if self.peek() == "<"
                && self
                    .tokens
                    .get(self.position + 1)
                    .is_some_and(|type_value| name_allowed(type_value.text))
            {
                return None;
            }

            self.next();
            let current_right = self.expression(precedence + 1)?;
            let span = SourceSpan::join(current_left.span, current_right.span);
            current_left = Expression {
                span,
                value_type: Type::Unknown,
                kind: ExpressionKind::BinaryOperator {
                    left: Box::new(current_left),
                    operator,
                    right: Box::new(current_right),
                },
            };
        }

        if self.peek() == "?" {
            return None;
        }

        Some(current_left)
    }

    fn prefix(&mut self) -> Attempt<Expression> {
        let start = self.current().start;
        let operator = match self.peek() {
            "-" => Some(UnaryOperator::Negate),
            "!" => Some(UnaryOperator::LogicalNot),
            "~" => Some(UnaryOperator::BitwiseNot),
            "++" => Some(UnaryOperator::PreIncrement),
            "--" => Some(UnaryOperator::PreDecrement),
            _ => None,
        };

        if let Some(operator) = operator {
            self.next();
            let expression = Box::new(self.prefix()?);

            return Some(Expression {
                span: Some(SourceSpan::new(start, self.current().start)),
                value_type: Type::Unknown,
                kind: ExpressionKind::UnaryOperator {
                    operator,
                    expression,
                },
            });
        }

        self.atom()
    }

    fn unary(&mut self) -> Attempt<Expression> {
        let start = self.current().start;
        let mut expression = self.prefix()?;

        if matches!(self.peek(), "[" | "." | "::" | "?") {
            return None;
        }

        let mut casts = 0;

        while self.eat("as") {
            casts += 1;
            if casts > 1 {
                return None;
            }

            let to = self.value_type()?;
            let span = SourceSpan::join(
                expression.span,
                Some(SourceSpan::new(start, self.current().start)),
            );
            expression = Expression {
                span,
                value_type: to.clone(),
                kind: ExpressionKind::Cast {
                    expression: Box::new(expression),
                    to,
                },
            };
        }

        Some(expression)
    }

    fn atom(&mut self) -> Attempt<Expression> {
        let token = self.next();
        let text = token.text;
        let (value_type, kind, span) = if text.as_bytes().first().is_some_and(u8::is_ascii_digit) {
            {
                let (value_type, kind) = numeric(text)?;

                (value_type, kind, SourceSpan::new(token.start, token.end))
            }
        } else if text == "true" || text == "false" {
            (
                Type::Boolean,
                ExpressionKind::Boolean(text == "true"),
                SourceSpan::new(token.start, token.end),
            )
        } else if text.starts_with('"') {
            let content = text.strip_prefix('"')?.strip_suffix('"')?;
            let decoded = decode_string(content)?;
            let parts = if decoded.is_empty() {
                Vec::new()
            } else {
                vec![StringPart::Text(decoded)]
            };

            (
                Type::String,
                ExpressionKind::String(StringLiteral { parts }),
                SourceSpan::new(token.start, token.end),
            )
        } else if text == "(" {
            let inner_expression = self.expression(0)?;
            self.need(")")?;

            return Some(inner_expression);
        } else if name_allowed(text) {
            if text.starts_with("true") || text.starts_with("false") {
                return None;
            }

            let mut call_path = vec![x_ast::CallSegment {
                name: text.to_owned(),
                type_arguments: Vec::new(),
            }];

            while self.eat("::") {
                call_path.push(x_ast::CallSegment {
                    name: self.name()?,
                    type_arguments: Vec::new(),
                });
            }

            let path_end = self.current().start;
            let raw = self.input.get(token.start..path_end)?;

            if self.eat("(") {
                let mut arguments = Vec::new();

                while self.peek() != ")" {
                    arguments.push(self.expression(0)?);
                    if !self.eat(",") {
                        break;
                    }

                    if self.peek() == ")" {
                        return None;
                    }
                }

                self.need(")")?;
                let end = self.tokens.get(self.position.checked_sub(1)?)?.end;

                (
                    Type::Unknown,
                    ExpressionKind::FunctionCall {
                        target: x_ast::CallTarget::new(x_ast::CallPath {
                            segments: call_path,
                        }),
                        arguments,
                    },
                    SourceSpan::new(token.start, end),
                )
            } else {
                (
                    Type::Unknown,
                    ExpressionKind::Identifier(raw.trim().to_owned()),
                    SourceSpan::new(token.start, path_end),
                )
            }
        } else {
            return None;
        };

        Some(Expression {
            span: Some(span),
            value_type,
            kind,
        })
    }
}

fn binary(text: &str) -> Attempt<(u8, Operator)> {
    Some(match text {
        "||" => (1, Operator::Or),
        "&&" => (2, Operator::And),
        "|" => (3, Operator::BitOr),
        "^" => (4, Operator::Xor),
        "&" => (5, Operator::BitAnd),
        "==" => (6, Operator::Equal),
        "!=" => (6, Operator::NotEqual),
        "<" => (7, Operator::LessThan),
        ">" => (7, Operator::GreaterThan),
        "<=" => (7, Operator::LessThanOrEqual),
        ">=" => (7, Operator::GreaterThanOrEqual),
        "<<" => (8, Operator::ShiftLeft),
        ">>" => (8, Operator::ShiftRight),
        "+" => (9, Operator::Add),
        "-" => (9, Operator::Subtract),
        "*" => (10, Operator::Multiply),
        "/" => (10, Operator::Divide),
        "%" => (10, Operator::Modulo),
        _ => return None,
    })
}
