use crate::errors::TypeError;
use crate::{FunctionSignature, TypeChecker};
use x_ast::SourceError;
use x_ast::StringLiteral;
use x_ast::StringPart;
use x_ast::intrinsics::{Intrinsic, IntrinsicScope};
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::frontend_messages::FrontendMessage;

impl TypeChecker {
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
