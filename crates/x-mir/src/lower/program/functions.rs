use super::MirLowerer;
use crate::errors::MirLowerError;
use crate::lower::type_contains_type_parameter;
use crate::source_names::type_to_source_name;
use crate::{CallContract, FunctionMeta, GlobalItem, MemoiseCallContract};
use x_ast::{GenericParameter, Parameter, Type};
use x_hir::{HirExternFunction, HirFunction, HirFunctionFlags};

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
