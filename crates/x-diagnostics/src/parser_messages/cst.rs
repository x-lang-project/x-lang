use std::fmt;

use crate::Error;

pub fn cst_missing_message(offset: usize, parent: impl fmt::Debug, expected: &str) -> String {
    Error::new(format!("invalid concrete syntax tree at byte {offset}: `{parent:?}` is missing {expected}; reduce the input and report this diagnostic")).render_detail()
}

pub fn cst_unexpected_message(
    offset: usize,
    parent: impl fmt::Debug,
    expected: &str,
    actual: impl fmt::Debug,
) -> String {
    Error::new(format!("invalid concrete syntax tree at byte {offset}: `{parent:?}` requires {expected}, found `{actual:?}`; reduce the input and report this diagnostic")).render_detail()
}

pub fn cst_unsupported_message(offset: usize, rule: impl fmt::Debug, detail: &str) -> String {
    Error::new(format!(
        "unsupported `{rule:?}` syntax at byte {offset}: {detail}"
    ))
    .render_detail()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CstMessage {
    AliasName,
    AliasTargetType,
    AlternatingOperandsAndOperators,
    ArrayRepetitionCount,
    ArrayRepetitionValue,
    AssociatedTypeName,
    UnsupportedAttributeTarget,
    BaseType,
    BindingPattern,
    BindingTypeOrValue,
    Block,
    BlockReturnExpressionOrExpression,
    CallablePath,
    ConditionExpressionAndBlock,
    ConditionExpressionAndThenBlock,
    ElseBlockOrNestedIf,
    EnumVariant,
    ErrorExpression,
    DuplicateExternBlockAbi,
    UnnamedExternBlockParameter,
    ExternalFunctionDeclarations,
    ExternalFunctionName,
    ExternalFunctionParameterList,
    ExternalParameter,
    FieldInitializers,
    FieldName,
    FieldType,
    FunctionName,
    Identifier,
    ForEachParts,
    ForRangeParts,
    ImplementationTargetType,
    InnerNode,
    InnerPattern,
    IntegerOrIdentifierRepetitionCount,
    InvariantBlock,
    AssignmentParts,
    MatchArmBody,
    MatchArms,
    NonEmptyImportPath,
    OddLengthOperandOperatorChain,
    LvalueParts,
    PostfixParts,
    PrimaryLvalueParts,
    Operand,
    ParameterList,
    ParameterName,
    Pattern,
    PostfixOperation,
    PostfixOperations,
    UnaryParts,
    PrefixParts,
    PrimaryExpression,
    UnsupportedWideIntegerLiteral,
    UnsupportedWideInteger,
    ReturnExpression,
    ScrutineeExpression,
    SelfOrNamedParameter,
    StatementNode,
    StructName,
    TargetUnaryExpression,
    InvalidTernaryArity,
    TraitName,
    TraitOrInherentImplDeclaration,
    TraitPath,
    TypeCasts,
    TypeName,
    UnaryExpression,
    UnaryOperand,
}
impl CstMessage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AliasName => "an alias name",
            Self::AliasTargetType => "an alias target type",
            Self::AlternatingOperandsAndOperators => "alternating operands and operators",
            Self::ArrayRepetitionCount => "an array repetition count",
            Self::ArrayRepetitionValue => "an array repetition value",
            Self::AssociatedTypeName => "an associated type name",
            Self::UnsupportedAttributeTarget => {
                "attributes are supported only for built-in function memoisation and struct layout or derives"
            }
            Self::BaseType => "a base type",
            Self::BindingPattern => "a binding pattern",
            Self::BindingTypeOrValue => "a binding type or value",
            Self::Block => "a block",
            Self::BlockReturnExpressionOrExpression => "a block, return expression, or expression",
            Self::CallablePath => "a callable path",
            Self::ConditionExpressionAndBlock => "a condition expression and block",
            Self::ConditionExpressionAndThenBlock => "a condition expression and then block",
            Self::ElseBlockOrNestedIf => "an else block or nested if",
            Self::EnumVariant => "an enum variant",
            Self::ErrorExpression => "an error expression",
            Self::DuplicateExternBlockAbi => "an extern block has at most one ABI name",
            Self::UnnamedExternBlockParameter => {
                "extern block parameters require explicit names and types"
            }
            Self::ExternalFunctionDeclarations => "external function declarations",
            Self::ExternalFunctionName => "an external function name",
            Self::ExternalFunctionParameterList => "an external function parameter list",
            Self::ExternalParameter => "an external parameter",
            Self::FieldInitializers => "field initialisers",
            Self::FieldName => "a field name",
            Self::FieldType => "a field type",
            Self::FunctionName => "a function name",
            Self::Identifier => "an identifier",
            Self::ForEachParts => "an identifier, iterator expression, and block",
            Self::ForRangeParts => "an identifier, range start, range end, and block",
            Self::ImplementationTargetType => "an implementation target type",
            Self::InnerNode => "an inner node",
            Self::InnerPattern => "an inner pattern",
            Self::IntegerOrIdentifierRepetitionCount => "an integer or identifier repetition count",
            Self::InvariantBlock => "an invariant block",
            Self::AssignmentParts => "an lvalue, assignment operator, and value",
            Self::MatchArmBody => "a match arm body",
            Self::MatchArms => "match arms",
            Self::NonEmptyImportPath => "a non-empty import path",
            Self::OddLengthOperandOperatorChain => "an odd-length operand/operator chain",
            Self::LvalueParts => "one address, dereference, or primary lvalue",
            Self::PostfixParts => "one array, call, method, try, field, or cast postfix",
            Self::PrimaryLvalueParts => "one postfix or parenthesised lvalue",
            Self::Operand => "an operand",
            Self::ParameterList => "a parameter list",
            Self::ParameterName => "a parameter name",
            Self::Pattern => "a pattern",
            Self::PostfixOperation => "a postfix operation",
            Self::PostfixOperations => "postfix operations",
            Self::UnaryParts => "a prefix expression, address, dereference, or postfix expression",
            Self::PrefixParts => "a prefix operator followed by a unary expression",
            Self::PrimaryExpression => "a primary expression",
            Self::UnsupportedWideIntegerLiteral => {
                "128-bit integer literals are reserved until the AST and backend represent them exactly"
            }
            Self::UnsupportedWideInteger => {
                "128-bit integers are reserved until the AST and backend represent them exactly"
            }
            Self::ReturnExpression => "a return expression",
            Self::ScrutineeExpression => "a scrutinee expression",
            Self::SelfOrNamedParameter => "a self or named parameter",
            Self::StatementNode => "a statement node",
            Self::StructName => "a struct name",
            Self::TargetUnaryExpression => "a target unary expression",
            Self::InvalidTernaryArity => {
                "a ternary expression must have one condition or condition/then/else"
            }
            Self::TraitName => "a trait name",
            Self::TraitOrInherentImplDeclaration => "a trait or inherent impl declaration",
            Self::TraitPath => "a trait path",
            Self::TypeCasts => "type casts",
            Self::TypeName => "a type name",
            Self::UnaryExpression => "a unary expression",
            Self::UnaryOperand => "a unary operand",
        }
    }
}
