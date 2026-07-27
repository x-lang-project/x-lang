use super::{FunctionInput, MirLowerer};
use crate::errors::MirLowerError;
use crate::lower::type_contains_type_parameter;
use crate::source_names::type_to_source_name;
use crate::{CallContract, FunctionMeta, GlobalItem, MemoiseCallContract};
use x_ast::{GenericParameter, Parameter, Type};
use x_hir::{HirExternFunction, HirFunction, HirFunctionFlags};

pub(super) fn has_codegen_generic_parameters(generic_parameters: &[GenericParameter]) -> bool {
    generic_parameters
        .iter()
        .any(|parameter| !matches!(parameter, GenericParameter::Lifetime { .. }))
}

pub(super) fn has_codegen_generic_signature(parameters: &[Parameter], return_type: &Type) -> bool {
    parameters
        .iter()
        .any(|parameter| type_contains_type_parameter(&parameter.value_type))
        || type_contains_type_parameter(return_type)
}

pub(super) fn lower_hir_function(
    lowerer: &mut MirLowerer,
    function: HirFunction,
) -> Result<Option<GlobalItem>, MirLowerError> {
    let Some(body) = function.body else {
        return Ok(None);
    };

    let HirFunctionFlags {
        pure: is_pure,
        memoised: is_memoised,
        is_static,
        multi: is_multi,
        throws: is_throws,
    } = function.flags;
    let meta = FunctionMeta {
        is_exported: function.is_exported,
        is_pure,
        is_memoised,
        call_contract: memoise_call_contract(is_memoised),
        is_static,
        is_multi,
        is_throws,
        has_static_parameters: function
            .parameters
            .iter()
            .any(|parameter| parameter.is_static),
    };

    lowerer
        .lower_function(FunctionInput {
            name: function.name,
            parameters: function.parameters,
            return_type: function.return_type,
            body,
            meta,
        })
        .map(Some)
}

pub(super) fn memoise_call_contract(is_memoised: bool) -> Option<CallContract> {
    is_memoised.then(|| {
        CallContract::Memoise(MemoiseCallContract::checked(
            crate::DEFAULT_MEMOISE_CAPACITY,
        ))
    })
}

pub(super) fn impl_method_owner_name(value_type: &Type) -> Option<String> {
    match value_type {
        Type::Custom(name) => Some(name.clone()),
        Type::GenericInstance {
            name,
            type_arguments,
        } => {
            if type_arguments.iter().all(is_erased_generic_owner_argument) {
                return Some(name.clone());
            }

            Some(type_to_source_name(value_type))
        }
        Type::I8
        | Type::I16
        | Type::I32
        | Type::I64
        | Type::ISize
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::USize
        | Type::F32
        | Type::F64
        | Type::Boolean
        | Type::Character
        | Type::String => Some(type_to_source_name(value_type)),
        _ => None,
    }
}

pub(super) fn is_erased_generic_owner_argument(value_type: &Type) -> bool {
    matches!(value_type, Type::TypeParameter(_))
}

pub(super) fn impl_owner_has_erased_generic_arguments(value_type: &Type) -> bool {
    match value_type {
        Type::GenericInstance { type_arguments, .. } => {
            type_arguments.iter().any(is_erased_generic_owner_argument)
                || type_arguments.iter().any(type_contains_type_parameter)
        }
        _ => type_contains_type_parameter(value_type),
    }
}
