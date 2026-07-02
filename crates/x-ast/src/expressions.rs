use std::fmt;

use crate::{
    CallTarget, Operator, SourceSpan, Statement, StructInitialisation, Type, UnaryOperator,
};

#[derive(Clone, PartialEq)]
pub struct Expression {
    pub span: Option<SourceSpan>,
    pub value_type: Type,
    pub kind: ExpressionKind,
}

impl fmt::Debug for Expression {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Expression")
            .field("value_type", &self.value_type)
            .field("kind", &self.kind)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Integer(i64),
    Float(f64),
    String(StringLiteral),
    Character(char),
    Boolean(bool),
    Identifier(String),
    TypeLiteral(Type),
    SizeOf(Type),
    Cast {
        expression: Box<Expression>,
        to: Type,
    },
    Tuple(Vec<Expression>),
    BinaryOperator {
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
    },
    FunctionCall {
        target: CallTarget,
        arguments: Vec<Expression>,
    },
    MethodCall {
        object: Box<Expression>,
        target: CallTarget,
        arguments: Vec<Expression>,
    },
    Try {
        expression: Box<Expression>,
    },
    AnonymousFunction {
        parameters: Vec<String>,
        body: Vec<Statement>,
        return_type: Option<Type>,
    },
    Array(Vec<Expression>),
    ArrayRepeat {
        value: Box<Expression>,
        count: Box<Expression>,
    },
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    Assignment {
        target: Box<Expression>,
        value: Box<Expression>,
    },
    StructInstantiate(StructInitialisation),
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },
    UnaryOperator {
        operator: UnaryOperator,
        expression: Box<Expression>,
    },
    AddressOf {
        is_mut: bool,
        is_unique: bool,
        expression: Box<Expression>,
    },
    Dereference {
        expression: Box<Expression>,
    },
    Assembly {
        source: String,
    },

    Block {
        statements: Vec<Statement>,
        result: Option<Box<Expression>>,
    },
    If {
        condition: Box<Expression>,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
    Match {
        scrutinee: Box<Expression>,
        arms: Vec<MatchArm>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Vec<Statement>,
    pub result: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringLiteral {
    pub parts: Vec<StringPart>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Text(String),
    Interpolation(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard,
    Identifier(String),
    Tuple(Vec<String>),
    Enum {
        variant: String,
        bindings: Vec<String>,
    },
}
