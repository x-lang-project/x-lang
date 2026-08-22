mod registered;

mod generic;
mod overloads;
mod resolution;
mod specialization;

use super::*;
use crate::FunctionDefinition;
use crate::MultiFunctionCandidate;
use crate::purity::ImpureOperation;
use crate::symbols::qualified_call_path_matches;
use x_ast::DeclarationKey;
use x_ast::GenericParameter;
use x_ast::SourceError;
use x_ast::intrinsics;
use x_ast::intrinsics::{Intrinsic, LayoutIntrinsic};
use x_diagnostics::frontend_messages::FrontendMessage;

impl TypeChecker {
    pub(super) fn check_function_call(
        &mut self,
        target: &mut x_ast::CallTarget,
        arguments: &mut [Expression],
    ) -> Result<CheckedType, TypeError> {
        // Consume the handler before checking argument calls.
        self.with_call_handler(|checker, handled| {
            checker.check_function_call_with_handler(target, arguments, handled)
        })
    }

    fn check_function_call_with_handler(
        &mut self,
        target: &mut x_ast::CallTarget,
        arguments: &mut [Expression],
        handled: bool,
    ) -> Result<CheckedType, TypeError> {
        let name = target.symbol_name().into_owned();
        let base_name = intrinsics::function_leaf_name(&name).to_owned();

        let resolved_name = self.symbols.resolve_function_call_name(
            &name,
            &base_name,
            self.current_function_name().map(String::as_str),
        );

        if let Some(signature) = self.symbols.get_function(&resolved_name).cloned() {
            return self
                .check_registered_function_call(
                    target,
                    arguments,
                    resolved_name,
                    signature,
                    handled,
                )
                .map(CheckedType::ResolveAnnotation);
        }

        if let Some((multi_name, candidates)) =
            self.symbols.resolve_multi_candidates(&name, &base_name)
        {
            return self
                .check_multi_call(target, arguments, multi_name, &candidates, handled)
                .map(CheckedType::ResolveAnnotation);
        }

        if let Some((generic_name, generic_function)) =
            self.symbols.resolve_generic_function(&name, &base_name)
        {
            return self
                .check_generic_function_call(
                    target,
                    arguments,
                    generic_name,
                    &generic_function,
                    handled,
                )
                .map(CheckedType::ResolveAnnotation);
        }

        // Source declarations take precedence over compiler intrinsic names.
        if !self.symbols.reserved_source_names.contains(name.as_str())
            && let Some(category) = intrinsics::early_impure_call_effect(&name)
        {
            self.require_impure_operation(ImpureOperation::BuiltinCall {
                name: &name,
                effect: category,
            })?;
        }

        if intrinsics::is_catch_call(&name) {
            return self
                .check_catch_call(arguments)
                .map(CheckedType::PreserveAnnotation);
        }

        if let Some(signature) = Self::intrinsic_signature(&name) {
            return self
                .check_intrinsic_call(target, arguments, signature)
                .map(CheckedType::ResolveAnnotation);
        }

        Err(TypeError::UnknownIdentifier(name))
    }

    fn check_intrinsic_call(
        &mut self,
        target: &mut x_ast::CallTarget,
        current_arguments: &mut [Expression],
        signature: FunctionSignature,
    ) -> Result<Type, TypeError> {
        let name = target.symbol_name().into_owned();
        let path = intrinsics::strip_explicit_type_arguments(&name).to_owned();
        let definition = Self::intrinsic_definition(&name);

        if matches!(
            definition,
            Some(Intrinsic::Layout(
                LayoutIntrinsic::SizeOf | LayoutIntrinsic::AlignOf
            ))
        ) {
            let arguments = target.leaf_type_arguments();
            let [argument] = arguments else {
                return Err(TypeError::ArityMismatch {
                    expected: 1,
                    got: arguments.len(),
                    symbol_name: format!("{path} type arguments"),
                });
            };

            let argument = self.resolve_type(argument)?;
            target.resolve_builtin_to(path, vec![argument]);
        }

        if definition == Some(Intrinsic::Layout(LayoutIntrinsic::OffsetOf)) {
            return self.check_offset_query(target, current_arguments);
        }

        if current_arguments.len() != signature.parameter_types.len() {
            return Err(TypeError::ArityMismatch {
                expected: signature.parameter_types.len(),
                got: current_arguments.len(),
                symbol_name: name.to_string(),
            });
        }

        if let Some(category) = intrinsics::impure_builtin_effect(&name) {
            self.require_impure_operation(ImpureOperation::BuiltinCall {
                name: &name,
                effect: category,
            })?;
        }

        for (current_argument, expected_type) in
            current_arguments.iter_mut().zip(&signature.parameter_types)
        {
            let actual_type = self.check_expression(current_argument)?;

            self.unify(expected_type, &actual_type)
                .map_err(|error| error.at_source(current_argument.span))?;

            if is_integer_literal(current_argument) && self.is_integer_primitive(expected_type) {
                current_argument.value_type = self.resolve(expected_type);
            }
        }

        Ok(signature.return_type)
    }

    fn check_catch_call(&mut self, arguments: &mut [Expression]) -> Result<Type, TypeError> {
        let got = arguments.len();
        let [call, fallback] = arguments else {
            return Err(TypeError::ArityMismatch {
                expected: 2,
                got,
                symbol_name: "catch".to_string(),
            });
        };

        if !matches!(call.kind, ExpressionKind::FunctionCall { .. }) {
            return Err(TypeError::inference(FrontendMessage::IndirectCatchCall));
        }

        if !matches!(
            fallback.kind,
            ExpressionKind::Integer(_)
                | ExpressionKind::Float(_)
                | ExpressionKind::Boolean(_)
                | ExpressionKind::Character(_)
                | ExpressionKind::String(_)
                | ExpressionKind::Identifier(_)
        ) && !matches!(
            &fallback.kind,
            ExpressionKind::UnaryOperator { operator: UnaryOperator::Negate, expression }
                if matches!(expression.kind, ExpressionKind::Integer(_) | ExpressionKind::Float(_))
        ) {
            return Err(TypeError::inference(
                FrontendMessage::UnevaluatedCatchFallback,
            ));
        }

        let call_type = self.with_handled_call_operand(|checker| checker.check_expression(call))?;
        let ExpressionKind::FunctionCall {
            target: called_name,
            ..
        } = &call.kind
        else {
            return Err(TypeError::inference(FrontendMessage::ChangedCatchOperand));
        };

        let called_name = called_name.symbol_name();
        let called_signature = self.symbols.get_function(&called_name).ok_or_else(|| {
            TypeError::inference(FrontendMessage::MissingCatchAbi {
                called_name: called_name.to_string(),
            })
        })?;

        if !called_signature.is_throws {
            return Err(TypeError::inference(
                FrontendMessage::NonThrowingCatchCall {
                    called_name: called_name.to_string(),
                },
            ));
        }

        let fallback_type = self.check_expression(fallback)?;
        self.unify(&call_type, &fallback_type)?;
        call.value_type = self.resolve(&call_type);
        fallback.value_type = self.resolve(&call_type);

        Ok(self.resolve(&call_type))
    }
}
