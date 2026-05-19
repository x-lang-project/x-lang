use super::*;

impl FrontendMessage {
    pub fn display_diagnostic(&self) -> crate::Diagnostic {
        let message: borrow::Cow<'_, str> = match self {
            Self::MissingDeclaredTypeArgument { parameter, owner } => format!("cannot infer declared parameter {parameter} for {owner}").into(),
            Self::StaticParameterCapturedByNestedFunction => "static parameters cannot be captured by a nested function".into(),
            Self::StaticParameterShadowedByLoop => "a loop binding cannot shadow a static parameter".into(),
            Self::StaticParameterShadowedByConstant => "a local constant cannot shadow a static parameter".into(),
            Self::AmbiguousMultiCall {
                multi_name,
                actual_types,
                candidates,
                ..
            } => format!(
                "ambiguous multi call '{multi_name}' for argument types {actual_types}; candidates: {candidates}"
            ).into(),
            Self::AmbiguousTrait {
                trait_name,
                candidates,
                ..
            } => format!(
                "ambiguous trait `{trait_name}`; candidates: {candidates}"
            ).into(),
            Self::NonConstantArrayRepetition => "array repetition length must be a compile-time integer literal".into(),
            Self::InvalidArrayRepetitionLength => "array repetition length must be non-negative and fit usize".into(),
            Self::BecomeAbiMismatch {
                callee,
                caller,
                caller_parameters,
                caller_return_type,
                callee_parameters,
                callee_return_type,
                ..
            } => format!(
                "`become {callee}` requires an ABI-identical signature; caller `{caller}` is ({caller_parameters}) -> {caller_return_type} but callee is ({callee_parameters}) -> {callee_return_type}"
            ).into(),
            Self::IndirectBecomeCall => "`become` requires a direct function call".into(),
            Self::BecomeOutsideFunction => "`become` is only valid inside a function".into(),
            Self::UnpropagatedThrowingCall { resolved_name, .. } => format!(
                "call to throwing function `{resolved_name}` must use postfix `?`"
            ).into(),
            Self::MissingBecomeCallerAbi { caller, .. } => format!(
                "cannot establish the calling convention for `become` in `{caller}`"
            ).into(),
            Self::MissingBecomeCalleeAbi { callee, .. } => format!(
                "cannot establish the calling convention for `become {callee}`"
            ).into(),
            Self::MissingThrowingAbi { name, .. } => format!("cannot establish a throws ABI for `{name}?`").into(),
            Self::MissingCatchAbi { called_name, .. } => format!(
                "cannot establish a throws ABI for catch({called_name}(...), ...)"
            ).into(),
            Self::MissingPropagationCaller { enclosing, .. } => format!(
                "cannot resolve enclosing function `{enclosing}` for postfix `?`"
            ).into(),
            Self::MissingThrowCaller { current, .. } => format!(
                "cannot resolve enclosing function `{current}` for `throw`"
            ).into(),
            Self::UnevaluatedCatchFallback => "`catch` fallback must be a literal or an already-evaluated value".into(),
            Self::ChangedCatchOperand => "`catch` operand changed shape during type checking".into(),
            Self::IndirectCatchCall => "`catch` requires a direct throwing call as its first argument".into(),
            Self::NonThrowingCatchCall { called_name, .. } => format!(
                "`catch` requires a throwing call, but `{called_name}` is non-throwing"
            ).into(),
            Self::DuplicateFieldInitializer {
                field_name,
                struct_name,
                ..
            } => format!(
                "duplicate field `{field_name}` in `{struct_name}` initialiser"
            ).into(),
            Self::DuplicateMultiOverload {
                name, parameter_types, ..
            } => format!(
                "duplicate multi overload `{name}` for parameter types {parameter_types}"
            ).into(),
            Self::UnsupportedMultiDeclaration { name } => format!(
                "multi function `{name}` must be a concrete top-level function; generic, trait and impl overloads are not supported"
            ).into(),
            Self::UnboundEmptyArray => "Empty array literals must carry type bindings".into(),
            Self::UnsupportedExternParameter {
                name,
                parameter_index,
                value_type,
                ..
            } => format!(
                "extern `C` function `{name}` parameter {parameter_index} uses unsupported FFI type `{value_type}`"
            ).into(),
            Self::UnsupportedExternReturn { name, return_type, .. } => format!(
                "extern `C` function `{name}` uses unsupported return type `{return_type}`"
            ).into(),
            Self::UnsupportedExternAbi { name, found, .. } => format!(
                "extern function `{name}` declares {found}; X 1.0 supports only exactly one `extern \"C\"` ABI label"
            ).into(),
            Self::MissingAssociatedType {
                trait_name,
                type_name,
                name,
                ..
            } => format!(
                "impl `{trait_name}` for `{type_name}` is missing required associated type `{name}`"
            ).into(),
            Self::MissingTraitMethod {
                trait_name,
                type_name,
                method_name,
                ..
            } => format!(
                "impl `{trait_name}` for `{type_name}` is missing required method `{method_name}`"
            ).into(),
            Self::UnsupportedIncrementOperator => "increment and decrement operators are not supported; use an explicit assignment".into(),
            Self::StaticParameterShadowedByLocal => "a local binding cannot shadow a static parameter".into(),
            Self::StaticParameterShadowedByMatch => "a match binding cannot shadow a static parameter".into(),
            Self::MissingMethodReceiver {
                struct_name,
                method,
                ..
            } => format!(
                "method `{struct_name}::{method}` has no receiver parameter"
            ).into(),
            Self::NoMatchingMultiOverload {
                multi_name,
                actual_types,
                ..
            } => format!(
                "No multi overload for '{multi_name}' matches argument types {actual_types}"
            ).into(),
            Self::NonStructOffsetOf { target, .. } => format!(
                "offset_of requires a concrete structure, found `{target}`"
            ).into(),
            Self::OffsetOfTypeArity => "offset_of requires exactly one structure type argument".into(),
            Self::OffsetOfValueField => "offset_of requires a field name, not a value expression".into(),
            Self::IndirectPropagationCall => "postfix `?` requires a direct call to a `throws` function".into(),
            Self::NonThrowingPropagationCaller { enclosing, .. } => format!(
                "postfix `?` in `{enclosing}` requires the enclosing function to be marked `throws`"
            ).into(),
            Self::NonThrowingPropagationCall { name, .. } => format!(
                "postfix `?` requires a throwing call, but `{name}` is non-throwing"
            ).into(),
            Self::PropagationOutsideFunction => "postfix `?` is only valid inside a function marked `throws`".into(),
            Self::UninspectablePureExtern { name, .. } => format!(
                "pure extern function `{name}` has no inspectable body and cannot provide LLVM optimisation facts"
            ).into(),
            Self::UninspectablePureFunction { name, .. } => format!(
                "pure function `{name}` requires an inspectable source body"
            ).into(),
            Self::MutablePureParameter {
                fully_qualified,
                name,
                ..
            } => format!(
                "pure function `{fully_qualified}` cannot accept mutable, unique, or raw-memory parameter `{name}`"
            ).into(),
            Self::SpecialisationSymbolSpaceExhausted => "specialisation symbol space exhausted".into(),
            Self::NonLiteralStaticArgument {
                parameter,
                function,
                ..
            } => format!(
                "static argument `{parameter}` to `{function}` must be a literal compile-time value"
            ).into(),
            Self::RuntimeHandleStaticParameter {
                name,
                fully_qualified,
                ..
            } => format!(
                "static parameter `{name}` in `{fully_qualified}` cannot be a reference or raw pointer"
            ).into(),
            Self::ChangedStaticSpecialisationQueue => "static-specialisation queue changed while rewriting an item".into(),
            Self::NonFunctionStaticTemplate => "static specialisation template is not a function".into(),
            Self::NonThrowingThrowCaller { current, .. } => format!(
                "`throw` in `{current}` requires the enclosing function to be marked `throws`"
            ).into(),
            Self::ThrowOutsideFunction => "`throw` is only valid inside a function marked `throws`".into(),
            Self::TuplePatternArityMismatch => "tuple pattern arity mismatch".into(),
            Self::NonTuplePattern => "tuple pattern requires a tuple".into(),
            Self::UnresolvedExpressionType { kind, .. } => format!(
                "cannot infer an expression type for {kind}"
            ).into(),
            Self::UnsatisfiedTraitBound {
                concrete_type,
                name,
                bound_name,
                ..
            } => format!(
                "type `{concrete_type}` does not satisfy trait bound `{name}: {bound_name}`"
            ).into(),
            Self::UnknownStruct { name, .. } => format!("unknown struct `{name}`").into(),
            Self::UnknownStructNamed { struct_name, .. } => format!("unknown struct `{struct_name}`").into(),
            Self::UnknownTrait { trait_name, .. } => format!("unknown trait `{trait_name}`").into(),
};

        Error::new(message)
    }
}
