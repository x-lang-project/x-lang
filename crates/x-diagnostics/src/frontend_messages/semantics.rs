use std::borrow;
use std::error;
use std::fmt;

use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontendMessage {
    MissingDeclaredTypeArgument {
        parameter: String,
        owner: String,
    },
    StaticParameterCapturedByNestedFunction,
    StaticParameterShadowedByLoop,
    StaticParameterShadowedByConstant,
    AmbiguousMultiCall {
        multi_name: String,
        actual_types: String,
        candidates: String,
    },
    AmbiguousTrait {
        trait_name: String,
        candidates: String,
    },
    NonConstantArrayRepetition,
    InvalidArrayRepetitionLength,
    BecomeAbiMismatch {
        callee: String,
        caller: String,
        caller_parameters: String,
        caller_return_type: String,
        callee_parameters: String,
        callee_return_type: String,
    },
    IndirectBecomeCall,
    BecomeOutsideFunction,
    UnpropagatedThrowingCall {
        resolved_name: String,
    },
    MissingBecomeCallerAbi {
        caller: String,
    },
    MissingBecomeCalleeAbi {
        callee: String,
    },
    MissingThrowingAbi {
        name: String,
    },
    MissingCatchAbi {
        called_name: String,
    },
    MissingPropagationCaller {
        enclosing: String,
    },
    MissingThrowCaller {
        current: String,
    },
    UnevaluatedCatchFallback,
    ChangedCatchOperand,
    IndirectCatchCall,
    NonThrowingCatchCall {
        called_name: String,
    },
    DuplicateFieldInitializer {
        field_name: String,
        struct_name: String,
    },
    DuplicateMultiOverload {
        name: String,
        parameter_types: String,
    },
    UnsupportedMultiDeclaration {
        name: String,
    },
    UnboundEmptyArray,
    UnsupportedExternParameter {
        name: String,
        parameter_index: usize,
        value_type: String,
    },
    UnsupportedExternReturn {
        name: String,
        return_type: String,
    },
    UnsupportedExternAbi {
        name: String,
        found: String,
    },
    MissingAssociatedType {
        trait_name: String,
        type_name: String,
        name: String,
    },
    MissingTraitMethod {
        trait_name: String,
        type_name: String,
        method_name: String,
    },
    UnsupportedIncrementOperator,
    StaticParameterShadowedByLocal,
    StaticParameterShadowedByMatch,
    MissingMethodReceiver {
        struct_name: String,
        method: String,
    },
    NoMatchingMultiOverload {
        multi_name: String,
        actual_types: String,
    },
    NonStructOffsetOf {
        target: String,
    },
    OffsetOfTypeArity,
    OffsetOfValueField,
    IndirectPropagationCall,
    NonThrowingPropagationCaller {
        enclosing: String,
    },
    NonThrowingPropagationCall {
        name: String,
    },
    PropagationOutsideFunction,
    UninspectablePureExtern {
        name: String,
    },
    UninspectablePureFunction {
        name: String,
    },
    MutablePureParameter {
        fully_qualified: String,
        name: String,
    },
    SpecialisationSymbolSpaceExhausted,
    NonLiteralStaticArgument {
        parameter: String,
        function: String,
    },
    RuntimeHandleStaticParameter {
        name: String,
        fully_qualified: String,
    },
    ChangedStaticSpecialisationQueue,
    NonFunctionStaticTemplate,
    NonThrowingThrowCaller {
        current: String,
    },
    ThrowOutsideFunction,
    TuplePatternArityMismatch,
    NonTuplePattern,
    UnresolvedExpressionType {
        kind: String,
    },
    UnsatisfiedTraitBound {
        concrete_type: String,
        name: String,
        bound_name: String,
    },
    UnknownStruct {
        name: String,
    },
    UnknownStructNamed {
        struct_name: String,
    },
    UnknownTrait {
        trait_name: String,
    },
}

impl fmt::Display for FrontendMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.display_diagnostic().render_detail())
    }
}

impl error::Error for FrontendMessage {}

mod render;
