use std::collections::HashMap;

use crate::*;
use x_ast::DeclarationKey;
use x_ast::SourceError;
use x_ast::SourceSpan;
use x_ast::{ImplDefinition, Program, Statement};

fn lower_item(statement: Statement) -> Result<Option<HirItem>, HirLowerError> {
    let item = match statement {
        Statement::Function {
            ref generic_parameters,
            ref parameters,
            ref return_type,
            ..
        } if generic_parameters
            .iter()
            .any(|parameter| !matches!(parameter, GenericParameter::Lifetime { .. }))
            || parameters
                .iter()
                .any(|parameter| type_contains_codegen_parameter(&parameter.value_type))
            || type_contains_codegen_parameter(return_type) =>
        {
            // Lower concrete monomorphs instead of generic templates.
            return Ok(None);
        }
        Statement::Function { .. } => HirItem::Function(lower_function(statement)?),
        Statement::ExternFunctionDeclaration {
            name,
            parameters,
            return_type,
            abi,
            is_pure,
        } => HirItem::ExternFunction(HirExternFunction {
            name,
            parameters,
            return_type,
            abi,
            is_pure,
        }),
        Statement::StructDeclaration(definition) => HirItem::Struct(definition),
        Statement::TraitDeclaration(definition) => HirItem::Trait(definition),
        Statement::ImplDeclaration(definition) => HirItem::Impl(lower_impl(definition)?),
        Statement::TypeAlias {
            name,
            generic_parameters,
            value_type,
        } => HirItem::TypeAlias {
            name,
            generic_parameters,
            value_type,
        },
        Statement::VariableDeclaration {
            pattern,
            type_annotation,
            value,
        } => HirItem::GlobalLet(HirLet {
            pattern,
            type_annotation,
            value,
        }),
        Statement::Constant {
            name,
            type_annotation,
            value,
        } => HirItem::Const {
            name,
            value_type: type_annotation,
            value,
        },
        Statement::Import { module, item } => HirItem::Import { module, item },
        Statement::Comment(_) => return Ok(None),
        other => return Err(HirLowerError::UnsupportedStatement(format!("{other:?}"))),
    };

    Ok(Some(item))
}

fn lower_impl(definition: ImplDefinition) -> Result<HirImpl, HirLowerError> {
    let mut methods = Vec::new();

    for mut method in definition.methods {
        x_ast::substitute_self_in_statement(&mut method, &definition.type_name);
        methods.push(lower_function(method)?);
    }

    Ok(HirImpl {
        generic_parameters: definition.generic_parameters,
        trait_name: definition.trait_name,
        type_name: definition.type_name,
        associated_types: definition.associated_types,
        methods,
    })
}

fn lower_function(statement: Statement) -> Result<HirFunction, HirLowerError> {
    let Statement::Function {
        name,
        generic_parameters,
        parameters,
        return_type,
        body,
        is_pure,
        is_memoised,
        is_static,
        is_multi,
        is_throws,
        is_exported,
    } = statement
    else {
        return Err(HirLowerError::NonFunctionImplItem(format!("{statement:?}")));
    };

    Ok(HirFunction {
        name,
        generic_parameters,
        parameters,
        return_type,
        is_exported,
        flags: HirFunctionFlags {
            pure: is_pure,
            memoised: is_memoised,
            is_static,
            multi: is_multi,
            throws: is_throws,
        },
        body: body.map(|body| *body),
    })
}
