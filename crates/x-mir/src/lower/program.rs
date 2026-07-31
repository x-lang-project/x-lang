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

pub fn lower_program(program: HirProgram) -> Result<MirProgram, MirLowerError> {
    let sources = &program.declaration_spans;
    let mut lowerer = MirLowerer::new();
    let mut items = Vec::new();

    for item in &program.items {
        match item {
            HirItem::Function(function) => {
                lowerer.source_functions.insert(function.name.clone());
            }
            HirItem::ExternFunction(function) => {
                lowerer.source_functions.insert(function.name.clone());
            }
            _ => {}
        }
    }

    let reachable_functions = compute_reachable_functions(&program);

    for item in &program.items {
        let origin = hir_declaration_key(item).and_then(|key| sources.get(&key).copied());

        if let Some(global) =
            lower_global_item(&mut lowerer, item).map_err(|error| error.at_source(origin))?
        {
            items.push(global);
        }
    }

    for item in program.items {
        let origin = hir_declaration_key(&item).and_then(|key| sources.get(&key).copied());

        match item {
            HirItem::Function(function) => {
                if has_codegen_generic_parameters(&function.generic_parameters)
                    || has_codegen_generic_signature(&function.parameters, &function.return_type)
                    || (!reachable_functions.is_empty()
                        && !is_reachable_source_function(&reachable_functions, &function.name))
                {
                    continue;
                }

                if let Some(lowered) = lower_hir_function(&mut lowerer, function)
                    .map_err(|error| error.at_source(origin))?
                {
                    items.push(lowered);
                }
            }
            HirItem::ExternFunction(function) => items.push(lower_external_function(function)),
            HirItem::Impl(impl_definition) => {
                let Some(owner_name) = impl_method_owner_name(&impl_definition.type_name) else {
                    continue;
                };

                if has_codegen_generic_parameters(&impl_definition.generic_parameters)
                    || impl_owner_has_erased_generic_arguments(&impl_definition.type_name)
                {
                    continue;
                }

                for mut method in impl_definition.methods {
                    method.name = format!("{owner_name}::{}", method.name);
                    let is_reachable =
                        is_reachable_source_function(&reachable_functions, &method.name);

                    if !reachable_functions.is_empty() && !is_reachable {
                        continue;
                    }

                    if !is_reachable
                        && (has_codegen_generic_parameters(&method.generic_parameters)
                            || has_codegen_generic_signature(
                                &method.parameters,
                                &method.return_type,
                            ))
                    {
                        continue;
                    }

                    if let Some(lowered) = lower_hir_function(&mut lowerer, method)
                        .map_err(|error| error.at_source(origin))?
                    {
                        items.push(lowered);
                    }
                }
            }
            _ => {}
        }
    }

    let mut program = MirProgram { items };
    program.resolve_call_targets();
    program.preserve_invariant_reference_contexts()?;

    Ok(program)
}

pub(crate) fn infer_tuple_pattern_value_type(
    value: &Expression,
    arity: usize,
) -> Result<Type, MirLowerError> {
    resolved_tuple_type(value, arity)
        .ok_or_else(|| MirLowerError::UnsupportedExpression(mir_messages::tuple_binding_arity()))
}
