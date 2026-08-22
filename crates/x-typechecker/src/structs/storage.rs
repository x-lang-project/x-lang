use std::collections::HashSet;

use super::*;

impl TypeChecker {
    pub(crate) fn validate_record_storage_cycles(&mut self) -> Result<(), errors::TypeError> {
        // Expand aliases before detecting by-value storage cycles.
        let mut normalized = HashSet::new();

        loop {
            let pending = self
                .symbols
                .records
                .fields()
                .keys()
                .filter(|name| !normalized.contains(*name))
                .cloned()
                .collect::<Vec<_>>();

            if pending.is_empty() {
                break;
            }

            for name in pending {
                normalized.insert(name.clone());
                let fields = self
                    .symbols
                    .records
                    .fields()
                    .get(&name)
                    .cloned()
                    .ok_or_else(|| {
                        errors::TypeError::inference(
                            x_diagnostics::frontend_messages::FrontendMessage::UnknownStruct {
                                name: name.clone(),
                            },
                        )
                    })?;
                let fields = fields
                    .into_iter()
                    .map(|(field, value_type)| {
                        self.resolve_type(&value_type)
                            .map(|value_type| (field, value_type))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.symbols
                    .records
                    .replace_fields(&name, fields)
                    .ok_or_else(|| {
                        errors::TypeError::inference(
                            x_diagnostics::frontend_messages::FrontendMessage::UnknownStruct {
                                name,
                            },
                        )
                    })?;
            }
        }

        fn nested_records(value_type: &Type, names: &mut Vec<String>) {
            match value_type {
                Type::Custom(name) => names.push(name.clone()),
                Type::FixedSizedArray(inner, _) | Type::Simd { element: inner, .. } => {
                    nested_records(inner, names)
                }
                Type::Tuple(elements) => elements
                    .iter()
                    .for_each(|value_type| nested_records(value_type, names)),
                _ => {}
            }
        }
        fn visit(
            checker: &TypeChecker,
            name: &str,
            active: &mut HashSet<String>,
            complete: &mut HashSet<String>,
        ) -> Result<(), errors::TypeError> {
            if complete.contains(name) {
                return Ok(());
            }

            let Some(fields) = checker.symbols.records.fields().get(name) else {
                return Ok(());
            };

            if !active.insert(name.to_string()) {
                return Err(errors::TypeError::ExecutableType {
                    value_type: Type::Custom(name.to_string()),
                    reason: x_diagnostics::frontend_messages::ExecutableTypeReason::RecursiveStorageCycle,
                });
            }

            if active.len() > 64 {
                return Err(errors::TypeError::ResourceLimit {
                    resource: crate::errors::TypeResource::RecordStorageDepth,
                    maximum: 64,
                });
            }

            for (_, value_type) in fields {
                let mut children = Vec::new();
                nested_records(value_type, &mut children);

                for child in children {
                    visit(checker, &child, active, complete)?;
                }
            }

            active.remove(name);
            complete.insert(name.to_string());

            Ok(())
        }
        let mut complete = HashSet::new();

        for name in self.symbols.records.fields().keys() {
            visit(self, name, &mut HashSet::new(), &mut complete)?;
        }

        Ok(())
    }
}
