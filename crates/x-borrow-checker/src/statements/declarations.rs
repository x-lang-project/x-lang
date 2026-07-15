use super::*;

impl BorrowChecker {
    pub(crate) fn check_function_statement(
        &mut self,
        parameters: &[x_ast::Parameter],
        return_type: &Type,
        body: Option<&[Statement]>,
        generic_parameters: &[x_ast::GenericParameter],
    ) -> Result<(), BorrowCheckError> {
        if has_non_lifetime_generic_parameter(generic_parameters) {
            return Ok(());
        }

        if let Some(body_vec) = body {
            let mut parameter_references = HashMap::new();
            let parameter_values = parameters
                .iter()
                .map(|current_parameter| {
                    parameter_origins::parameter(
                        &current_parameter.name,
                        &current_parameter.value_type,
                        &self.context.struct_layouts,
                        &mut parameter_references,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;

            let enclosing_termination = self.flow.path_terminated;
            let enclosing_exits = mem::take(&mut self.flow.loop_exits);

            self.flow.path_terminated = false;
            std::sync::Arc::make_mut(&mut self.context)
                .function_contexts
                .push(FunctionLifetimeContext {
                    parameter_references,
                    return_type: return_type.clone(),
                });
            let result = self.with_scope(|this| {
                for (current_parameter, references) in parameters.iter().zip(parameter_values) {
                    this.declare_variable(
                        &current_parameter.name,
                        current_parameter.value_type.clone(),
                        true,
                    )?;

                    if let Some((binding, _)) = this.lookup_variable_mut(&current_parameter.name) {
                        binding.references = references;
                    }
                }

                this.check_statements(body_vec)
            });

            std::sync::Arc::make_mut(&mut self.context)
                .function_contexts
                .pop();
            self.flow.path_terminated = enclosing_termination;
            self.flow.loop_exits = enclosing_exits;
            result?;
        }

        Ok(())
    }

    pub(crate) fn check_variable_statement(
        &mut self,
        pattern: &Pattern,
        type_annotation: &Option<Type>,
        value: &x_ast::Expression,
    ) -> Result<(), BorrowCheckError> {
        let mut references = self.check_transferred_value(value)?;

        let is_initialised = !matches!(value.kind, ExpressionKind::TypeLiteral(_));
        let determined_type = type_annotation.clone().unwrap_or_else(|| {
            if value.value_type == Type::Unknown
                && let ExpressionKind::Identifier(name) = &value.kind
            {
                self.lookup_variable(name)
                    .map(|(binding, _)| binding.value_type.clone())
                    .unwrap_or(Type::Unknown)
            } else {
                value.value_type.clone()
            }
        });

        if is_initialised && !self.flow.path_terminated {
            self.validate_value_lifetimes(&determined_type, &references, "initialiser")?;
        }

        if let Pattern::Identifier(name) = pattern {
            let mut variable_info = VariableInfo::new(determined_type, is_initialised);
            variable_info.references = references;

            if let ExpressionKind::AddressOf {
                is_mut,
                is_unique,
                expression,
            } = &value.kind
            {
                let is_unique_borrow = *is_unique
                    || matches!(
                        type_annotation,
                        Some(Type::Reference {
                            is_unique: true,
                            ..
                        })
                    );
                variable_info.value_type = Type::Reference {
                    lifetime: declared_reference_lifetime(type_annotation.as_ref()),
                    is_mut: *is_mut,
                    is_unique: is_unique_borrow,
                    inner: Box::new(expression.value_type.clone()),
                };
            }

            if type_annotation.is_none()
                && let ExpressionKind::Identifier(source) = &value.kind
                && let Some((source_variable_info, _)) = self.lookup_variable(source)
                && matches!(source_variable_info.value_type, Type::Reference { .. })
            {
                variable_info.value_type = source_variable_info.value_type.clone();
            }

            let scope = self
                .flow
                .scopes
                .last_mut()
                .ok_or(BorrowCheckError::Invariant(
                    borrow_messages::BorrowInvariant::UnallocatedScopeContext,
                ))?;
            scope.insert(name.clone(), variable_info);

            return Ok(());
        }

        if let Pattern::Tuple(names) = pattern {
            let element_types = match &determined_type {
                Type::Tuple(elements) => elements.as_slice(),
                _ => &[],
            };

            let scope = self
                .flow
                .scopes
                .last_mut()
                .ok_or(BorrowCheckError::Invariant(
                    borrow_messages::BorrowInvariant::UnallocatedScopeContext,
                ))?;

            for (index, nm) in names.iter().enumerate() {
                let value_type = element_types.get(index).cloned().unwrap_or(Type::Unknown);
                let mut binding = VariableInfo::new(value_type, is_initialised);
                binding.references = references
                    .fields
                    .remove(&ReferenceProjection::Index(index as i64))
                    .unwrap_or_default();
                scope.insert(nm.clone(), binding);
            }

            return Ok(());
        }

        Ok(())
    }

    pub(crate) fn check_return_statement(
        &mut self,
        value: Option<&x_ast::Expression>,
    ) -> Result<(), BorrowCheckError> {
        if let Some(element) = value {
            let references = self.check_transferred_value(element)?;

            if self.is_reference_expression(element)? && !self.flow.path_terminated {
                self.validate_reference_return(element, &references)?;
            }

            if !self.flow.path_terminated
                && let Some(current_context) = self.context.function_contexts.last()
            {
                self.validate_returned_value(&current_context.return_type, &references)?;
            }

            if let Some(target) = references.find_target(|target| target.origin_region != Region(0))
            {
                return Err(BorrowCheckError::LifetimeTooShort {
                    target: target.target_variable.clone(),
                });
            }
        }
        self.flow.path_terminated = true;

        Ok(())
    }
}
