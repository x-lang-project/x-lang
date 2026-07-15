use std::collections::HashMap;

use crate::*;
use x_ast::DeclarationKey;
use x_ast::SourceError;
use x_ast::SourceSpan;
use x_ast::{ImplDefinition, Program, Statement};

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
