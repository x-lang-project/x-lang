use std::collections::{HashMap, HashSet};

use super::CheckedType;
use crate::purity::{ImpureOperation, LocalProjection};
use crate::{TypeChecker, errors::TypeError};
use x_ast::StructInitialisation;
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::frontend_messages::FrontendMessage;

impl TypeChecker {
    pub(super) fn check_identifier(&mut self, name: &mut String) -> Result<CheckedType, TypeError> {
        if let Some((resolved_name, variable_type, is_global)) = self
            .symbols
            .resolve_variable_with_scope(name, self.current_function_name().map(String::as_str))
        {
            if is_global
                && self.current_function_is_pure()
                && !self.symbols.is_global_constant(&resolved_name)
            {
                self.require_impure_operation(ImpureOperation::GlobalRead(&resolved_name))?;
            }

            if resolved_name != *name {
                *name = resolved_name;
            }

            return Ok(CheckedType::Annotate(variable_type));
        }

        let mut resolved_name = name.clone();
        let mut found_global = false;

        if let Some(aliased_path) = self.symbols.imported_aliases.get(name) {
            if self.symbols.get_variable(aliased_path).is_some() {
                resolved_name = aliased_path.clone();
                found_global = true;
            }
        } else if let Some(current_function) = self.current_function_name()
            && let Some(namespace) = x_ast::intrinsics::function_owner_name(current_function)
        {
            let prefixed_candidate = format!("{namespace}::{name}");

            if self.symbols.get_variable(&prefixed_candidate).is_some() {
                resolved_name = prefixed_candidate;
                found_global = true;
            }
        }

        if found_global {
            if self.current_function_is_pure() && !self.symbols.is_global_constant(&resolved_name) {
                self.require_impure_operation(ImpureOperation::GlobalRead(&resolved_name))?;
            }

            let variable_type = self
                .symbols
                .get_variable(&resolved_name)
                .cloned()
                .ok_or_else(|| TypeError::UnknownIdentifier(resolved_name.clone()))?;
            *name = resolved_name;

            return Ok(CheckedType::Annotate(variable_type));
        }

        if let Some(function_signature) = self.symbols.get_function(name) {
            Ok(CheckedType::ResolveAnnotation(Type::Function {
                parameters: function_signature.parameter_types.clone(),
                return_value: Box::new(function_signature.return_type.clone()),
            }))
        } else {
            Err(TypeError::UnknownIdentifier(name.clone()))
        }
    }
}

pub(super) fn find_declared_field<'a>(
    layout: &'a [(String, Type)],
    requested: &str,
) -> Option<(&'a String, &'a Type)> {
    layout
        .iter()
        .find(|(name, _)| name == requested)
        .map(|(name, value_type)| (name, value_type))
}
