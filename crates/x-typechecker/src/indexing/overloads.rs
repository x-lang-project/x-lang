use super::*;

impl TypeChecker {
    pub(super) fn index_multi_function(
        &mut self,
        name: &str,
        signature: FunctionSignature,
    ) -> Result<(), TypeError> {
        let normalized = |checker: &mut Self, types: &[Type]| -> Result<Vec<Type>, TypeError> {
            struct EraseLifetimes;
            impl x_ast::TypeVisitorMut for EraseLifetimes {
                fn visit_type(&mut self, value_type: &mut Type) {
                    match value_type {
                        Type::Reference {
                            lifetime, inner, ..
                        } => {
                            *lifetime = None;
                            self.visit_type(inner);
                        }
                        Type::RawPointer { inner, .. }
                        | Type::Array(inner)
                        | Type::FixedSizedArray(inner, _)
                        | Type::Simd { element: inner, .. } => self.visit_type(inner),
                        Type::Tuple(items)
                        | Type::GenericInstance {
                            type_arguments: items,
                            ..
                        } => {
                            for item in items {
                                self.visit_type(item);
                            }
                        }
                        Type::Function {
                            parameters,
                            return_value,
                        } => {
                            for parameter in parameters {
                                self.visit_type(parameter);
                            }

                            self.visit_type(return_value);
                        }
                        _ => {}
                    }
                }
            }
            let mut resolved = types
                .iter()
                .map(|value_type| checker.resolve_type(value_type))
                .collect::<Result<Vec<_>, _>>()?;

            for value_type in &mut resolved {
                x_ast::TypeVisitorMut::visit_type(&mut EraseLifetimes, value_type);
            }

            Ok(resolved)
        };
        let new_types = normalized(self, &signature.parameter_types)?;
        let existing_signatures = self
            .symbols
            .multi_functions
            .get(name)
            .map(|candidates| {
                candidates
                    .iter()
                    .map(|character| character.signature.parameter_types.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        for existing in existing_signatures {
            if normalized(self, &existing)? == new_types {
                return Err(TypeError::inference(
                    FrontendMessage::DuplicateMultiOverload {
                        name: name.to_string(),
                        parameter_types: format!("{new_types:?}"),
                    },
                ));
            }
        }

        let mangled_name = self
            .symbols
            .vacant_generated_name(&Self::multi_mangled_name(name, &signature.parameter_types))
            .map_err(TypeError::Invariant)?;

        let candidates = self
            .symbols
            .multi_functions
            .entry(name.to_string())
            .or_default();

        if candidates
            .iter()
            .any(|candidate| candidate.signature.parameter_types == signature.parameter_types)
        {
            return Err(TypeError::inference(
                FrontendMessage::DuplicateMultiOverload {
                    name: name.to_string(),
                    parameter_types: format!("{:?}", signature.parameter_types),
                },
            ));
        }

        candidates.push(MultiFunctionCandidate {
            mangled_name: mangled_name.clone(),
            signature: signature.clone(),
        });
        self.symbols.add_function(mangled_name, signature);

        Ok(())
    }
}
