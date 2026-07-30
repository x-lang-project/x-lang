use std::mem;

mod freezing;

use std::collections::{HashMap, HashSet};

use super::{Contexts, Invariants, dependencies, rewrite, unsupported};
use crate::errors::MirLowerError;
use crate::{LocalIdentifier, MirBody, MirStatement, Operand, Place, Rvalue};
use x_ast::Type;

struct Builder<'a> {
    body: &'a mut MirBody,
    invariants: &'a Invariants,
    definitions: HashMap<LocalIdentifier, Vec<Rvalue>>,
    contexts: Contexts,
    visiting: HashSet<LocalIdentifier>,
    completed: HashSet<LocalIdentifier>,
    prefixes: HashMap<LocalIdentifier, Vec<MirStatement>>,
}

pub(super) fn prepare(
    body: &mut MirBody,
    invariants: &Invariants,
) -> Result<Contexts, MirLowerError> {
    let mut definitions: HashMap<LocalIdentifier, Vec<Rvalue>> = HashMap::new();

    for block in body.basic_blocks.values() {
        for statement in &block.statements {
            if let MirStatement::Assign(destination, value) = statement {
                if let Place::Local(identifier) = destination {
                    definitions
                        .entry(*identifier)
                        .or_default()
                        .push(value.clone());
                } else if let Rvalue::AddressOf {
                    is_mut: true,
                    expression,
                    ..
                } = value
                    && dependencies::needs_context(expression, body, invariants)
                {
                    return Err(unsupported(
                        "a dependent projection cannot be stored in an aggregate",
                    ));
                }
            }
        }
    }

    let mut identifiers: Vec<_> = definitions.keys().copied().collect();
    identifiers.sort_unstable_by_key(|identifier| identifier.0);
    let mut builder = Builder {
        body,
        invariants,
        definitions,
        contexts: HashMap::new(),
        visiting: HashSet::new(),
        completed: HashSet::new(),
        prefixes: HashMap::new(),
    };

    for identifier in identifiers {
        builder.context(identifier)?;
    }

    for block in builder.body.basic_blocks.values_mut() {
        let mut statements = Vec::new();

        for mut statement in mem::take(&mut block.statements) {
            if let MirStatement::Assign(
                Place::Local(identifier),
                Rvalue::AddressOf { expression, .. },
            ) = &mut statement
                && let Some(target) = builder.contexts.get(identifier)
            {
                statements.extend(builder.prefixes.remove(identifier).unwrap_or_default());
                *expression = target.clone();
            }

            statements.push(statement);
        }
        block.statements = statements;
    }

    Ok(builder.contexts)
}

impl Builder<'_> {
    fn context(&mut self, identifier: LocalIdentifier) -> Result<Option<Place>, MirLowerError> {
        if self.completed.contains(&identifier) {
            return Ok(self.contexts.get(&identifier).cloned());
        }

        if !self.visiting.insert(identifier) {
            return Ok(None);
        }

        let values = self
            .definitions
            .get(&identifier)
            .cloned()
            .unwrap_or_default();
        let mut candidates = Vec::new();

        for value in &values {
            if let Some(context) = self.value_context(value)? {
                candidates.push(context);
            }
        }

        self.visiting.remove(&identifier);
        if !candidates.is_empty() {
            if values.len() != 1 {
                return Err(unsupported(
                    "dependent reference rebinding or branch-selected origins are unsupported",
                ));
            }

            let value_type = self.body.place_type(&Place::Local(identifier));

            if !matches!(value_type, Type::Reference { .. }) {
                return Err(unsupported(
                    "dependent references cannot be cast to non-reference types",
                ));
            }

            if let Some(mut target) = candidates.into_iter().next() {
                if matches!(values.first(), Some(Rvalue::AddressOf { .. })) {
                    let mut prefix = Vec::new();
                    target = self.freeze(target, &mut prefix)?;
                    self.prefixes.insert(identifier, prefix);
                }

                self.contexts.insert(identifier, target);
            }
        }

        self.completed.insert(identifier);

        Ok(self.contexts.get(&identifier).cloned())
    }

    fn value_context(&mut self, value: &Rvalue) -> Result<Option<Place>, MirLowerError> {
        match value {
            Rvalue::Use(Operand::Copy(place) | Operand::Move(place)) => {
                if let Place::Local(identifier) = place.as_ref() {
                    self.context(*identifier)
                } else {
                    Ok(None)
                }
            }
            Rvalue::AddressOf {
                is_mut: true,
                expression,
                ..
            } => {
                let mut identifiers = Vec::new();
                rewrite::local_identifiers(expression, &mut identifiers);
                for identifier in identifiers {
                    self.context(identifier)?;
                }

                let target = rewrite::place(expression, &self.contexts);

                Ok(
                    dependencies::needs_context(&target, self.body, self.invariants)
                        .then_some(target),
                )
            }
            _ => Ok(None),
        }
    }
}
