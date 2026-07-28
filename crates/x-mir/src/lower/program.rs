mod functions;
mod globals;
mod reachability;

use super::{FunctionInput, MirLowerer, resolved_tuple_type};
use crate::MirProgram;
use crate::errors::MirLowerError;
use functions::{
    has_codegen_generic_parameters, has_codegen_generic_signature, impl_method_owner_name,
    impl_owner_has_erased_generic_arguments, lower_external_function, lower_hir_function,
};
use globals::lower_global_item;
use reachability::{compute_reachable_functions, is_reachable_source_function};
use x_ast::{Expression, Pattern, SourceError, Type};
use x_diagnostics::mir_messages;
use x_hir::{HirItem, HirLet, HirProgram};

fn hir_declaration_key(item: &HirItem) -> Option<x_ast::DeclarationKey> {
    use x_ast::DeclarationKey;

    match item {
        HirItem::Function(function) => Some(DeclarationKey::Function(
            function.name.clone(),
            function
                .parameters
                .iter()
                .map(|parameter| parameter.value_type.clone())
                .collect(),
        )),
        HirItem::Struct(definition) => Some(DeclarationKey::Struct(definition.name.clone())),
        HirItem::Impl(definition) => Some(DeclarationKey::Impl(
            definition.trait_name.clone(),
            definition.type_name.clone(),
        )),
        HirItem::GlobalLet(HirLet {
            pattern: Pattern::Identifier(name),
            ..
        }) => Some(DeclarationKey::Binding(vec![name.clone()])),
        HirItem::Const { name, .. } => Some(DeclarationKey::Constant(name.clone())),
        _ => None,
    }
}

pub(crate) fn infer_tuple_pattern_value_type(
    value: &Expression,
    arity: usize,
) -> Result<Type, MirLowerError> {
    resolved_tuple_type(value, arity)
        .ok_or_else(|| MirLowerError::UnsupportedExpression(mir_messages::tuple_binding_arity()))
}
