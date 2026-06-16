use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    Value(i32),
    Binary {
        operator: Operator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BooleanExpression {
    Value(bool),
    Comparison {
        operator: ComparisonOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Not(Box<BooleanExpression>),
    Binary {
        operator: BooleanOperator,
        left: Box<BooleanExpression>,
        right: Box<BooleanExpression>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BooleanOperator {
    And,
    Or,
}

impl Expression {
    /// Evaluate left to right; intermediate values must fit i32.
    #[must_use]
    pub fn evaluate(&self) -> i32 {
        match self {
            Self::Value(value) => *value,
            Self::Binary {
                operator,
                left,
                right,
            } => {
                let left = left.evaluate();
                let right = right.evaluate();

                operator.evaluate(left, right)
            }
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(value) => fmt::Display::fmt(value, formatter),
            Self::Binary {
                operator,
                left,
                right,
            } => {
                write!(formatter, "({left} {} {right})", operator.spelling())
            }
        }
    }
}

impl Operator {
    fn evaluate(self, left: i32, right: i32) -> i32 {
        match self {
            Self::Add => left + right,
            Self::Sub => left - right,
            Self::Mul => left * right,
            Self::Div => left / right,
            Self::Rem => left % right,
            Self::BitAnd => left & right,
            Self::BitOr => left | right,
            Self::BitXor => left ^ right,
            Self::ShiftLeft => left << right,
            Self::ShiftRight => left >> right,
        }
    }

    #[must_use]
    pub const fn spelling(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Rem => "%",
            Self::BitAnd => "&",
            Self::BitOr => "|",
            Self::BitXor => "^",
            Self::ShiftLeft => "<<",
            Self::ShiftRight => ">>",
        }
    }
}

impl BooleanExpression {
    #[must_use]
    pub fn render(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for BooleanExpression {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(value) => fmt::Display::fmt(value, formatter),
            Self::Comparison {
                operator,
                left,
                right,
            } => {
                write!(formatter, "({left} {} {right})", operator.spelling())
            }
            Self::Not(expression) => write!(formatter, "!({expression})"),
            Self::Binary {
                operator,
                left,
                right,
            } => {
                write!(formatter, "({left} {} {right})", operator.spelling())
            }
        }
    }
}

impl ComparisonOperator {
    fn evaluate(self, left: i32, right: i32) -> bool {
        match self {
            Self::Equal => left == right,
            Self::NotEqual => left != right,
            Self::Less => left < right,
            Self::LessEqual => left <= right,
            Self::Greater => left > right,
            Self::GreaterEqual => left >= right,
        }
    }

    #[must_use]
    pub const fn spelling(self) -> &'static str {
        match self {
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::Less => "<",
            Self::LessEqual => "<=",
            Self::Greater => ">",
            Self::GreaterEqual => ">=",
        }
    }
}

impl BooleanOperator {
    #[must_use]
    pub const fn spelling(self) -> &'static str {
        match self {
            Self::And => "&&",
            Self::Or => "||",
        }
    }
}
