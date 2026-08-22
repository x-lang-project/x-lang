mod substitution;

mod compatibility;

use std::collections::HashMap;

use crate::tail_helpers::qualified_path_suffix_matches;
use crate::{TypeAliasDefinition, TypeChecker, errors::TypeError, symbols::mangle_type_components};
use x_ast::Type;

impl TypeChecker {
    pub fn register_internal_helpers(&mut self) {
        let u8_pointer_mut = Type::RawPointer {
            is_mut: true,
            inner: Box::new(Type::U8),
        };
        self.symbols.add_function(
            "__x_write_u8".to_string(),
            crate::function_signature!(vec![u8_pointer_mut, Type::U8] => Type::Void),
        );
    }

    pub fn resolve_type(&mut self, value_type: &Type) -> Result<Type, TypeError> {
        self.with_type_resolution(
            crate::errors::TypeResource::TypeResolutionDepth,
            |checker| checker.resolve_type_inner(value_type),
        )
    }

    fn resolve_type_inner(&mut self, value_type: &Type) -> Result<Type, TypeError> {
        match value_type {
            Type::Infer(_) => {
                let resolved = self.resolve(value_type);

                if resolved != *value_type {
                    self.resolve_type(&resolved)
                } else {
                    Ok(value_type.clone())
                }
            }
            Type::Custom(name) => {
                if let Some(alias) = self.lookup_type_alias(name) {
                    return self.expand_type_alias(name, &alias, &[]);
                }

                if let Ok(resolved_name) = self.resolve_struct_name(name)
                    && resolved_name != *name
                {
                    return Ok(Type::Custom(resolved_name));
                }

                Ok(value_type.clone())
            }
            Type::GenericInstance {
                name,
                type_arguments,
            } => {
                let resolved_arguments = self.resolve_types(type_arguments)?;

                if let Some(alias) = self.lookup_type_alias(name) {
                    return self.expand_type_alias(name, &alias, &resolved_arguments);
                }

                if let Some(struct_key) = self.resolve_generic_struct_key(name) {
                    let mangled = self.instantiate_struct(&struct_key, &resolved_arguments)?;

                    return Ok(Type::Custom(mangled));
                }

                Ok(Type::GenericInstance {
                    name: name.clone(),
                    type_arguments: resolved_arguments,
                })
            }
            Type::Array(inner) => Ok(Type::Array(Box::new(self.resolve_type(inner)?))),
            Type::FixedSizedArray(inner, length) => Ok(Type::FixedSizedArray(
                Box::new(self.resolve_type(inner)?),
                *length,
            )),
            Type::Tuple(elements) => Ok(Type::Tuple(self.resolve_types(elements)?)),
            Type::Simd { element, lanes } => Ok(Type::Simd {
                element: Box::new(self.resolve_type(element)?),
                lanes: *lanes,
            }),
            Type::RawPointer { is_mut, inner } => Ok(Type::RawPointer {
                is_mut: *is_mut,
                inner: Box::new(self.resolve_type(inner)?),
            }),
            Type::Reference {
                lifetime,
                is_mut,
                is_unique,
                inner,
            } => Ok(Type::Reference {
                lifetime: lifetime.clone(),
                is_mut: *is_mut,
                is_unique: *is_unique,
                inner: Box::new(self.resolve_type(inner)?),
            }),
            Type::Function {
                parameters,
                return_value,
            } => Ok(Type::Function {
                parameters: self.resolve_types(parameters)?,
                return_value: Box::new(self.resolve_type(return_value)?),
            }),
            _ => Ok(value_type.clone()),
        }
    }

    fn resolve_types(&mut self, types: &[Type]) -> Result<Vec<Type>, TypeError> {
        types
            .iter()
            .map(|value_type| self.resolve_type(value_type))
            .collect()
    }

    pub(crate) fn lookup_type_alias(&self, name: &str) -> Option<TypeAliasDefinition> {
        self.symbols
            .imported_aliases
            .get(name)
            .and_then(|key| self.symbols.type_aliases.get(key))
            .cloned()
            .or_else(|| self.symbols.type_aliases.get(name).cloned())
            .or_else(|| {
                self.current_namespace.as_ref().and_then(|namespace| {
                    self.symbols
                        .type_aliases
                        .get(&format!("{namespace}::{name}"))
                        .cloned()
                })
            })
            .or_else(|| {
                let mut matches = self
                    .symbols
                    .type_aliases
                    .iter()
                    .filter(|(alias_name, _)| qualified_alias_name_matches(alias_name, name));

                let first = matches.next()?;

                if matches.next().is_none() {
                    Some(first.1.clone())
                } else {
                    None
                }
            })
    }

    fn expand_type_alias(
        &mut self,
        name: &str,
        alias: &TypeAliasDefinition,
        arguments: &[Type],
    ) -> Result<Type, TypeError> {
        let parameters = alias
            .generic_parameters
            .iter()
            .filter_map(|parameter| match parameter {
                x_ast::GenericParameter::Type { name, default, .. } => Some((name, default)),
                _ => None,
            })
            .collect::<Vec<_>>();

        if arguments.len() > parameters.len() {
            return Err(TypeError::ArityMismatch {
                expected: parameters.len(),
                got: arguments.len(),
                symbol_name: name.into(),
            });
        }

        let expected = parameters.len();
        let mut substitutions = HashMap::new();

        for (index, (parameter, default)) in parameters.into_iter().enumerate() {
            let argument = arguments
                .get(index)
                .cloned()
                .or_else(|| default.clone())
                .ok_or_else(|| TypeError::ArityMismatch {
                    expected,
                    got: arguments.len(),
                    symbol_name: name.into(),
                })?;
            let argument = self.substitute_type(&argument, &substitutions);
            substitutions.insert(parameter.clone(), argument);
        }

        let expanded = self.substitute_type(&alias.value_type, &substitutions);

        self.resolve_type(&expanded)
    }
}

fn qualified_alias_name_matches(registered: &str, requested: &str) -> bool {
    qualified_path_suffix_matches(registered, requested)
}

pub(crate) fn lifetimes_compatible(
    expected: &Option<x_ast::Lifetime>,
    actual: &Option<x_ast::Lifetime>,
) -> bool {
    use x_ast::Lifetime;

    match (expected, actual) {
        (None, _) | (_, None) => true,
        (Some(Lifetime::Erased), _) | (_, Some(Lifetime::Erased)) => true,
        (Some(_), Some(Lifetime::Static)) => true,
        (Some(Lifetime::Named(expected)), Some(Lifetime::Named(actual))) => expected == actual,
        (Some(Lifetime::Infer(_)), _) | (_, Some(Lifetime::Infer(_))) => true,
        _ => false,
    }
}
