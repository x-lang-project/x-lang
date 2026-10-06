use operations::*;
use x_ast::{Layout, StructDefinition, Type};
use x_diagnostics::CompilationFailure;
use x_diagnostics::contract_messages;
use x_mir::{GlobalItem, MirBody, MirProgram, MirStatement, MirTerminator, Operand, Place, Rvalue};

pub(crate) fn validate(program: &MirProgram) -> Result<(), CompilationFailure> {
    for item in &program.items {
        validate_item(item)?;
    }

    Ok(())
}

fn reject(detail: String) -> Result<(), CompilationFailure> {
    Err(contract_messages::cheri_unsupported_construct(&detail))
}

fn validate_item(item: &GlobalItem) -> Result<(), CompilationFailure> {
    match item {
        GlobalItem::Function {
            name,
            parameter_types,
            return_type,
            body,
            ..
        } => validate_function(name, parameter_types, return_type, body.as_ref()),
        GlobalItem::Struct(definition) => validate_structure(definition),
        GlobalItem::Trait(definition) => {
            reject(contract_messages::cheri_trait_call(&definition.name))
        }
        GlobalItem::Impl { trait_name, .. } => {
            reject(contract_messages::cheri_trait_implementation(trait_name))
        }
        GlobalItem::Static { name, .. } => reject(contract_messages::cheri_static(name)),
    }
}

fn validate_function(
    name: &str,
    parameter_types: &[Type],
    return_type: &Type,
    body: Option<&MirBody>,
) -> Result<(), CompilationFailure> {
    for value_type in parameter_types {
        validate_type(
            value_type,
            &contract_messages::parameter_of_function_name(name),
        )?;
    }

    validate_type(
        return_type,
        &contract_messages::return_type_of_function_name(name),
    )?;
    let Some(body) = body else {
        return reject(contract_messages::cheri_external_function(name));
    };

    if body.is_throws {
        return reject(contract_messages::cheri_throwing_function(name));
    }

    for local in body.locals.values() {
        let local_context = match local.name.as_deref() {
            Some(local_name) => {
                contract_messages::local_local_name_in_function_name(local_name, name)
            }
            None => contract_messages::compiler_temporary_in_function_function(name),
        };

        validate_type(&local.value_type, &local_context)?;
    }

    for block in body.basic_blocks.values() {
        for statement in &block.statements {
            validate_statement(statement, name)?;
        }

        if let Some(terminator) = &block.terminator {
            validate_terminator(terminator, name)?;
        }
    }

    Ok(())
}

fn validate_structure(definition: &StructDefinition) -> Result<(), CompilationFailure> {
    if definition.layout != Layout::AoS {
        return reject(contract_messages::cheri_structure_layout(
            &definition.name,
            definition.layout,
        ));
    }

    if definition.generic_parameters.is_some() {
        return reject(contract_messages::cheri_generic_structure(&definition.name));
    }

    for (field, value_type) in &definition.fields {
        validate_type(
            value_type,
            &contract_messages::field_field_of_structure_value(field, &definition.name),
        )?;
    }

    Ok(())
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
