use operations::*;
use x_ast::{Layout, StructDefinition, Type};
use x_diagnostics::CompilationFailure;
use x_diagnostics::contract_messages;
use x_mir::{GlobalItem, MirBody, MirProgram, MirStatement, MirTerminator, Operand, Place, Rvalue};

fn reject(detail: String) -> Result<(), CompilationFailure> {
    Err(contract_messages::cheri_unsupported_construct(&detail))
}

fn validate_type(value_type: &Type, context: &str) -> Result<(), CompilationFailure> {
    match value_type {
        Type::I8
        | Type::U8
        | Type::I16
        | Type::U16
        | Type::I32
        | Type::U32
        | Type::I64
        | Type::U64
        | Type::ISize
        | Type::USize
        | Type::F32
        | Type::F64
        | Type::Character
        | Type::Boolean
        | Type::Void
        | Type::Custom(_) => Ok(()),
        Type::FixedSizedArray(inner, _) | Type::Reference { inner, .. } => {
            validate_type(inner, context)
        }
        Type::Array(_) => reject(contract_messages::dynamic_slice_in_context(context)),
        Type::String => reject(contract_messages::managed_string_in_context(context)),
        Type::RawPointer { .. } => reject(contract_messages::raw_pointer_in_context(context)),
        Type::Tuple(_) => reject(contract_messages::tuple_in_context(context)),
        Type::Simd { .. } => reject(contract_messages::simd_value_in_context(context)),
        Type::Function { .. } => reject(contract_messages::function_value_or_closure_in_context(
            context,
        )),
        Type::TypeParameter(_) | Type::GenericInstance { .. } => reject(
            contract_messages::unresolved_generic_type_in_context(context),
        ),
        Type::Never | Type::Unknown | Type::Infer(_) => {
            reject(contract_messages::unresolved_type_in_context(context))
        }
    }
}

mod operations;
