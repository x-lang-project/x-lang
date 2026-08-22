use crate::errors::TypeError;
use crate::{FunctionSignature, TypeChecker};
use x_ast::SourceError;
use x_ast::StringLiteral;
use x_ast::StringPart;
use x_ast::intrinsics::{Intrinsic, IntrinsicScope};
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::frontend_messages::FrontendMessage;

impl TypeChecker {
    pub(super) fn check_offset_query(
        &mut self,
        call: &mut x_ast::CallTarget,
        arguments: &mut [Expression],
    ) -> Result<Type, TypeError> {
        let type_arguments = call.leaf_type_arguments();
        let [type_argument] = type_arguments else {
            return Err(TypeError::inference(FrontendMessage::OffsetOfTypeArity));
        };

        let target = self.resolve_type(type_argument)?;
        let Type::Custom(structure) = &target else {
            return Err(TypeError::inference(FrontendMessage::NonStructOffsetOf {
                target: target.to_string(),
            }));
        };

        let structure = self.resolve_struct_name(structure)?;
        let [argument] = arguments else {
            return Err(TypeError::ArityMismatch {
                expected: 1,
                got: arguments.len(),
                symbol_name: call.symbol_name().into_owned(),
            });
        };

        let ExpressionKind::Identifier(field) = &argument.kind else {
            return Err(
                TypeError::inference(FrontendMessage::OffsetOfValueField).at_source(argument.span)
            );
        };

        if !self
            .symbols
            .records
            .fields()
            .get(&structure)
            .is_some_and(|fields| fields.iter().any(|(candidate, _)| candidate == field))
        {
            return Err(TypeError::UnresolvedProperty {
                field: field.clone(),
                value_type: Type::Custom(structure),
            }
            .at_source(argument.span));
        }

        argument.kind = ExpressionKind::String(StringLiteral {
            parts: vec![StringPart::Text(field.clone())],
        });
        argument.value_type = Type::String;
        call.resolve_builtin_to("offset_of".into(), vec![Type::Custom(structure)]);

        Ok(Type::USize)
    }

    pub(super) fn intrinsic_definition(name: &str) -> Option<Intrinsic> {
        let path = x_ast::intrinsics::strip_explicit_type_arguments(name);
        let base = match path {
            "mem::size_of" => "size_of",
            "mem::align_of" => "align_of",
            "mem::offset_of" => "offset_of",
            _ => path.strip_prefix("intrinsic::").unwrap_or(path),
        };

        Intrinsic::resolve(base, IntrinsicScope::Source)
    }

    pub(super) fn intrinsic_signature(name: &str) -> Option<FunctionSignature> {
        let signature = Self::intrinsic_definition(name)?.signature();

        Some(crate::function_signature!(signature.parameters => signature.result))
    }
}
