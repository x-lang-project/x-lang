use super::*;

impl TypeChecker {
    pub(super) fn check_declaration_initializer(
        &mut self,
        type_annotation: &mut Option<Type>,
        value: &mut Expression,
        describe_expected: impl FnOnce(String) -> String,
    ) -> Result<Type, TypeError> {
        if matches!(value.kind, ExpressionKind::StructInstantiate(_))
            && let Some(annotation) = type_annotation.as_ref()
        {
            value.value_type = self.resolve_type(annotation)?;
        }

        let empty_array_binding =
            matches!(&value.kind, ExpressionKind::Array(elements) if elements.is_empty())
                .then(|| {
                    type_annotation
                        .as_ref()
                        .map(|annotation| self.resolve_type(annotation))
                })
                .flatten()
                .transpose()?;
        let right_type = if let Some(annotation @ Type::FixedSizedArray(_, 0)) = empty_array_binding
        {
            annotation
        } else {
            self.check_expression(value)?
        };
        let declared_type = if let Some(current_annotation) = type_annotation {
            self.resolve_type(current_annotation)?
        } else {
            right_type.clone()
        };

        self.unify(&declared_type, &right_type)
            .map_err(|error| error.map_expected(describe_expected))?;
        let final_type = self.resolve(&declared_type);

        if matches!(&final_type, Type::Array(_)) {
            return Err(TypeError::ExecutableType {
                value_type: final_type,
                reason: ExecutableTypeReason::UnsizedArrayLength,
            });
        }

        if let Some(current_annotation) = type_annotation {
            *current_annotation = final_type.clone();
        }
        value.value_type = final_type.clone();

        Ok(final_type)
    }
}

impl TypeChecker {
    pub(super) fn check_variable_declaration(
        &mut self,
        pattern: &mut Pattern,
        type_annotation: &mut Option<Type>,
        value: &mut Expression,
    ) -> Result<(), TypeError> {
        let final_type =
            self.check_declaration_initializer(type_annotation, value, |expected| {
                format!("{expected} for variable declaration {pattern:?}")
            })?;
        // Resolve initialiser origins before introducing shadowing names.
        let local_reference = self.local_reference_paths(value);

        match pattern {
            Pattern::Identifier(name) => {
                self.symbols.add_variable(name.clone(), final_type);
                self.record_local_reference_paths(name, local_reference);
            }
            Pattern::Tuple(names) => {
                if let Type::Tuple(elements) = &final_type {
                    if names.len() != elements.len() {
                        return Err(TypeError::ArityMismatch {
                            expected: elements.len(),
                            got: names.len(),
                            symbol_name: "tuple pattern".to_string(),
                        });
                    }

                    for (index, (name, value_type)) in names.iter().zip(elements.iter()).enumerate()
                    {
                        self.symbols.add_variable(name.clone(), value_type.clone());
                        let prefix = [crate::purity::LocalProjection::TupleIndex(index)];
                        let paths = local_reference
                            .iter()
                            .filter_map(|path| {
                                path.strip_prefix(prefix.as_slice())
                                    .map(|rest| rest.to_vec())
                            })
                            .collect();
                        self.record_local_reference_paths(name, paths);
                    }
                } else {
                    return Err(TypeError::inference(FrontendMessage::NonTuplePattern));
                }
            }
            Pattern::Enum { .. } => {
                return Err(TypeError::ExecutableType {
                    value_type: final_type,
                    reason: ExecutableTypeReason::MissingEnumDiscriminantAbi,
                });
            }
            Pattern::Wildcard => {}
        }

        Ok(())
    }
}
