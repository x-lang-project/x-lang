use super::*;

impl TypeChecker {
    pub(super) fn check_multi_call(
        &mut self,
        target: &mut x_ast::CallTarget,
        arguments: &mut [Expression],
        multi_name: String,
        candidates: &[MultiFunctionCandidate],
        handled: bool,
    ) -> Result<Type, TypeError> {
        let mut actual_types = Vec::with_capacity(arguments.len());

        for argument in arguments.iter_mut() {
            let actual = self.check_expression(argument)?;
            actual_types.push(self.resolve_type(&actual)?);
        }

        let mut arity_matched = false;
        let mut matches = Vec::new();

        for candidate in candidates {
            let Ok(current_signature) =
                call_lifetimes::instantiate(&candidate.signature, &actual_types)
            else {
                continue;
            };

            if current_signature.parameter_types.len() != arguments.len() {
                continue;
            }

            arity_matched = true;

            let expected_types = current_signature
                .parameter_types
                .iter()
                .map(|value_type| self.resolve_type(value_type))
                .collect::<Result<Vec<_>, _>>()?;

            if actual_types
                .iter()
                .zip(&expected_types)
                .all(|(actual_type, expected_type)| {
                    self.multi_candidate_matches(expected_type, actual_type)
                })
            {
                matches.push(candidate);
            }
        }

        if !arity_matched {
            return Err(TypeError::ArityMismatch {
                expected: candidates
                    .first()
                    .map(|candidate| candidate.signature.parameter_types.len())
                    .unwrap_or(0),
                got: arguments.len(),
                symbol_name: multi_name,
            });
        }

        match matches.as_slice() {
            [] => Err(TypeError::lookup(
                FrontendMessage::NoMatchingMultiOverload {
                    multi_name: multi_name.to_string(),
                    actual_types: format!("{:?}", actual_types),
                },
            )),
            [candidate] => {
                if candidate.signature.is_throws && !handled {
                    return Err(TypeError::inference(
                        FrontendMessage::UnpropagatedThrowingCall {
                            resolved_name: candidate.mangled_name.clone(),
                        },
                    ));
                }

                self.check_pure_call(&candidate.mangled_name, &candidate.signature)?;

                for (argument, actual_type) in arguments.iter_mut().zip(actual_types.iter()) {
                    argument.value_type = self.resolve(actual_type);
                }

                target.resolve_to(candidate.mangled_name.clone());

                let signature = call_lifetimes::instantiate(&candidate.signature, &actual_types)?;

                Ok(self.resolve(&signature.return_type))
            }
            ambiguous => Err(TypeError::inference(FrontendMessage::AmbiguousMultiCall {
                multi_name: multi_name.to_string(),
                actual_types: format!("{:?}", actual_types),
                candidates: (ambiguous
                    .iter()
                    .map(|candidate| candidate.mangled_name.as_str())
                    .collect::<Vec<_>>()
                    .join(", "))
                .to_string(),
            })),
        }
    }
}
