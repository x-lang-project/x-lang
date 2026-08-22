mod construction;

use std::collections::{HashMap, HashSet};

use super::CheckedType;
use crate::purity::{ImpureOperation, LocalProjection};
use crate::{TypeChecker, errors::TypeError};
use x_ast::StructInitialisation;
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::frontend_messages::FrontendMessage;

impl TypeChecker {
    // MIR handles safe borrows; raw-pointer projections are checked here.
    pub(super) fn check_invariant_field_borrow(
        &mut self,
        target: &Expression,
    ) -> Result<(), TypeError> {
        let raw_owner = self.projection_has_raw_base(target)?;
        let mut projection = target;
        let mut field_path = Vec::new();

        loop {
            match &projection.kind {
                ExpressionKind::FieldAccess { object, field } => {
                    field_path.insert(0, LocalProjection::Field(field.clone()));
                    let receiver = self.resolve_type(&object.value_type)?;
                    let receiver = Self::unwrap_type_references(receiver);

                    if let Type::Custom(name) = receiver
                        && raw_owner
                        && self.symbols.invariant_structs.contains(&name)
                        && self
                            .symbols
                            .invariant_dependencies
                            .get(&name)
                            .and_then(Option::as_ref)
                            .is_none_or(|fields| {
                                fields.iter().any(|dependency| {
                                    // Parent and child projections overlap in either direction.
                                    dependency
                                        .iter()
                                        .zip(&field_path)
                                        .all(|(dependent, borrowed)| {
                                            matches!(borrowed, LocalProjection::Field(field) if dependent == field)
                                        })
                                })
                            })
                    {
                        return Err(TypeError::InvariantFieldBorrow { owner: name });
                    }

                    projection = object;
                }
                ExpressionKind::ArrayAccess { array, .. } => {
                    field_path.insert(0, LocalProjection::AnyIndex);
                    projection = array;
                }
                ExpressionKind::Dereference { expression } => projection = expression,
                _ => return Ok(()),
            }
        }
    }

    fn projection_has_raw_base(&mut self, target: &Expression) -> Result<bool, TypeError> {
        let mut projection = target;

        loop {
            let value_type = self.resolve_type(&projection.value_type)?;
            let mut value_type = &value_type;

            while let Type::Reference { inner, .. } = value_type {
                value_type = inner;
            }

            if matches!(value_type, Type::RawPointer { .. }) {
                return Ok(true);
            }

            projection = match &projection.kind {
                ExpressionKind::FieldAccess { object, .. } => object,
                ExpressionKind::ArrayAccess { array, .. } => array,
                ExpressionKind::Dereference { expression } => expression,
                _ => return Ok(false),
            };
        }
    }

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

    pub(super) fn check_field_access(
        &mut self,
        object: &mut Box<Expression>,
        field: &mut String,
    ) -> Result<CheckedType, TypeError> {
        let object_type = self.check_expression(object)?;
        let base_type = Self::unwrap_type_references(self.resolve(&object_type));
        let base_type = match &base_type {
            Type::Custom(name) | Type::GenericInstance { name, .. }
                if self.lookup_type_alias(name).is_some() =>
            {
                let resolved = self.resolve_type(&base_type)?;

                Self::unwrap_type_references(resolved)
            }
            _ => base_type,
        };

        let (struct_name, receiver_type_arguments) = match base_type {
            Type::Custom(struct_name) => (struct_name, Vec::new()),
            Type::GenericInstance {
                name,
                type_arguments,
            } => (name, type_arguments),
            _ => {
                return Err(TypeError::UnresolvedProperty {
                    field: field.clone(),
                    value_type: base_type,
                });
            }
        };

        let struct_name = self.resolve_struct_name(&struct_name)?;

        let (layout, generic_parameters) =
            if let Some(layout) = self.symbols.records.fields().get(&struct_name) {
                (layout, None)
            } else if let Some(sdef) = self.symbols.records.templates().get(&struct_name) {
                (&sdef.fields, sdef.generic_parameters.as_ref())
            } else {
                return Err(TypeError::lookup(FrontendMessage::UnknownStructNamed {
                    struct_name: struct_name.to_string(),
                }));
            };

        let Some((actual_field, field_type)) = find_declared_field(layout, field) else {
            return Err(TypeError::UnresolvedProperty {
                field: field.clone(),
                value_type: Type::Custom(struct_name),
            });
        };

        if actual_field != field {
            *field = actual_field.clone();
        }

        if receiver_type_arguments.is_empty() {
            Ok(CheckedType::ResolveAnnotation(field_type.clone()))
        } else {
            let mut substitutions = HashMap::new();

            if let Some(generic_parameters) = generic_parameters {
                for (parameter_name, argument_type) in generic_parameters
                    .iter()
                    .filter_map(x_ast::GenericParameter::non_lifetime_name)
                    .zip(receiver_type_arguments.iter())
                {
                    substitutions.insert(parameter_name.clone(), argument_type.clone());
                }
            }

            let field_type = field_type.clone();

            Ok(CheckedType::ResolveAnnotation(
                self.substitute_type(&field_type, &substitutions),
            ))
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
