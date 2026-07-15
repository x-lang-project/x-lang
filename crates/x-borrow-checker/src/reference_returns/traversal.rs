use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum OriginRequirement {
    DeclaredLifetime,
    Known,
    External,
    TailTransfer,
}

#[derive(Clone, Copy)]
pub(super) struct LifetimeTraversal {
    depth: usize,
    requirement: OriginRequirement,
}

impl LifetimeTraversal {
    pub(super) fn new(requirement: OriginRequirement) -> Self {
        Self {
            depth: 0,
            requirement,
        }
    }

    pub(super) fn descend(self) -> Self {
        Self {
            depth: self.depth + 1,
            ..self
        }
    }
}

impl BorrowChecker {
    pub(super) fn validate_nested_lifetimes(
        &self,
        value_type: &Type,
        value: &ReferenceValue,
        target: &str,
        traversal: LifetimeTraversal,
    ) -> Result<(), BorrowCheckError> {
        if traversal.depth >= MAX_AGGREGATE_LAYOUT_DEPTH {
            return Err(BorrowCheckError::Invariant(
                borrow_messages::BorrowInvariant::StoredAggregateLayoutDepthExceeded {
                    maximum: MAX_AGGREGATE_LAYOUT_DEPTH,
                },
            ));
        }

        let unknown = ReferenceValue::default();

        match value_type {
            Type::Reference {
                lifetime: Some(expected @ (Lifetime::Named(_) | Lifetime::Static)),
                ..
            } if traversal.requirement != OriginRequirement::TailTransfer => {
                if self
                    .context
                    .function_contexts
                    .last()
                    .is_some_and(|current_context| {
                        parameter_origins_satisfy(value, current_context, expected)
                    })
                {
                    Ok(())
                } else {
                    Err(BorrowCheckError::LifetimeTooShort {
                        target: target.into(),
                    })
                }
            }
            Type::Reference { .. } if traversal.requirement == OriginRequirement::Known => {
                if value.origin_known && !value.targets.is_empty() {
                    Ok(())
                } else {
                    Err(BorrowCheckError::UnknownReferenceOrigin {
                        target: target.into(),
                    })
                }
            }
            Type::Reference { .. } | Type::RawPointer { .. }
                if matches!(
                    traversal.requirement,
                    OriginRequirement::External | OriginRequirement::TailTransfer
                ) && (!matches!(value_type, Type::RawPointer { .. })
                    || traversal.requirement == OriginRequirement::TailTransfer) =>
            {
                if let Some(origin) = value
                    .targets
                    .iter()
                    .find(|origin| origin.origin_region != Region(0))
                {
                    return Err(BorrowCheckError::LifetimeTooShort {
                        target: origin.target_variable.clone(),
                    });
                }

                if value.origin_known
                    && !value.targets.is_empty()
                    && (traversal.requirement == OriginRequirement::TailTransfer
                        || self
                            .context
                            .function_contexts
                            .last()
                            .is_some_and(|current_context| {
                                value.targets.iter().all(|target| {
                                    target.origin_region == Region(0)
                                        && current_context
                                            .parameter_references
                                            .contains_key(&target.target_variable)
                                })
                            }))
                {
                    Ok(())
                } else {
                    Err(BorrowCheckError::InvalidReferenceReturn)
                }
            }
            Type::Tuple(items) => {
                for (index, value_type) in items.iter().enumerate() {
                    let field = ReferenceProjection::Index(index as i64);
                    self.validate_nested_lifetimes(
                        value_type,
                        value.fields.get(&field).unwrap_or(&unknown),
                        &format!("{target}.{field}"),
                        traversal.descend(),
                    )?;
                }

                Ok(())
            }
            Type::FixedSizedArray(_, 0) => Ok(()),
            Type::Array(inner) | Type::FixedSizedArray(inner, _) => {
                if value.fields.is_empty() {
                    self.validate_nested_lifetimes(inner, &unknown, target, traversal.descend())?;
                } else {
                    for (field, value) in &value.fields {
                        self.validate_nested_lifetimes(
                            inner,
                            value,
                            &format!("{target}[{field}]"),
                            traversal.descend(),
                        )?;
                    }
                }

                Ok(())
            }
            Type::Custom(_) | Type::GenericInstance { .. } => {
                if let Some(fields) =
                    parameter_origins::fields(value_type, &self.context.struct_layouts)
                {
                    for (field, value_type) in fields {
                        self.validate_nested_lifetimes(
                            &value_type,
                            value
                                .fields
                                .get(&ReferenceProjection::Field(field.clone()))
                                .unwrap_or(&unknown),
                            &format!("{target}.{field}"),
                            traversal.descend(),
                        )?;
                    }
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }
}
