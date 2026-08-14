use super::*;

impl TypeChecker {
    pub(crate) fn record_local_reference_paths(
        &mut self,
        name: &str,
        paths: HashSet<Vec<LocalProjection>>,
    ) {
        if let Some((key, _)) = self.local_binding(name) {
            self.pure_local_references.insert(key, paths);
        }
    }

    pub(super) fn local_storage_path(
        &self,
        expression: &Expression,
    ) -> Option<((usize, String), Vec<LocalProjection>)> {
        match &expression.kind {
            ExpressionKind::Identifier(name) => {
                self.local_binding(name).map(|(key, _)| (key, Vec::new()))
            }
            ExpressionKind::FieldAccess { object, field } => {
                if matches!(
                    self.resolve(&object.value_type),
                    Type::Reference { .. } | Type::RawPointer { .. }
                ) {
                    return None;
                }

                let (key, mut path) = self.local_storage_path(object)?;
                path.push(LocalProjection::Field(field.clone()));

                Some((key, path))
            }
            ExpressionKind::ArrayAccess { array, index } => {
                if matches!(
                    self.resolve(&array.value_type),
                    Type::Reference { .. } | Type::RawPointer { .. }
                ) {
                    return None;
                }

                let (key, mut path) = self.local_storage_path(array)?;
                path.push(match &index.kind {
                    ExpressionKind::Integer(position) => LocalProjection::Index(*position),
                    _ => LocalProjection::AnyIndex,
                });

                Some((key, path))
            }
            _ => None,
        }
    }

    // Array wildcards require the same provenance for every element.
    pub(crate) fn local_reference_paths(
        &self,
        value: &Expression,
    ) -> HashSet<Vec<LocalProjection>> {
        if let ExpressionKind::AddressOf {
            is_mut: true,
            expression,
            ..
        } = &value.kind
        {
            return if self.is_local_owned_assignment(expression)
                && !matches!(
                    self.resolve(&expression.value_type),
                    Type::Reference { .. } | Type::RawPointer { .. }
                ) {
                HashSet::from([Vec::new()])
            } else {
                HashSet::new()
            };
        }

        if let Some((key, prefix)) = self.local_storage_path(value) {
            return self
                .pure_local_references
                .get(&key)
                .into_iter()
                .flatten()
                .filter(|path| {
                    path.len() >= prefix.len()
                        && path
                            .iter()
                            .zip(&prefix)
                            .all(|(stored, projection)| stored.covers(projection))
                })
                .map(|path| path.iter().skip(prefix.len()).cloned().collect())
                .collect();
        }

        let mut paths = HashSet::new();
        let mut add = |prefix: LocalProjection, value: &Expression| {
            for mut path in self.local_reference_paths(value) {
                path.insert(0, prefix.clone());
                paths.insert(path);
            }
        };

        match &value.kind {
            ExpressionKind::StructInstantiate(initialise) => {
                for (field, value) in &initialise.fields {
                    add(LocalProjection::Field(field.clone()), value);
                }
            }
            ExpressionKind::Tuple(values) => {
                for (index, value) in values.iter().enumerate() {
                    add(LocalProjection::TupleIndex(index), value);
                }
            }
            ExpressionKind::Array(values) => {
                for (index, value) in values.iter().enumerate() {
                    if let Ok(index) = i64::try_from(index) {
                        add(LocalProjection::Index(index), value);
                    }
                }

                if let Some(first) = values.first() {
                    let mut common = self.local_reference_paths(first);

                    for value in values.iter().skip(1) {
                        let other = self.local_reference_paths(value);
                        common.retain(|path| other.contains(path));
                    }

                    for mut path in common {
                        path.insert(0, LocalProjection::AnyIndex);
                        paths.insert(path);
                    }
                }
            }
            ExpressionKind::ArrayRepeat { value, .. } => {
                add(LocalProjection::AnyIndex, value);
            }
            _ => {}
        }

        paths
    }

    pub(crate) fn update_local_reference_assignment(
        &mut self,
        target: &Expression,
        value: &Expression,
    ) {
        let Some((key, prefix)) = self.local_storage_path(target) else {
            return;
        };

        let new_paths = self.local_reference_paths(value);

        if let Some(paths) = self.pure_local_references.get_mut(&key) {
            // Intersect assignment facts because the branch may never execute.
            paths.retain(|path| {
                if let Some(rest) = path.strip_prefix(prefix.as_slice()) {
                    return new_paths.contains(rest);
                }

                !path
                    .iter()
                    .zip(&prefix)
                    .any(|(stored, changed)| stored.invalidated_by(changed))
            });
        }
    }
}
