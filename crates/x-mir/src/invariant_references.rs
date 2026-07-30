mod contexts;
mod dependencies;
mod escapes;
mod rewrite;

use std::collections::HashMap;

use crate::errors::MirLowerError;
use crate::{GlobalItem, LocalIdentifier, MirBody, MirProgram, Place};
use x_ast::StructInvariant;

type Contexts = HashMap<LocalIdentifier, Place>;
type Invariants = HashMap<String, StructInvariant>;

fn unsupported(detail: &str) -> MirLowerError {
    MirLowerError::InvalidContract(
        x_diagnostics::mir_messages::invariant_reference_context_required(detail),
    )
}

impl MirProgram {
    pub fn preserve_invariant_reference_contexts(&mut self) -> Result<(), MirLowerError> {
        let invariants = invariant_registry(&self.items);

        visit_bodies(&mut self.items, &mut |body| {
            let contexts = contexts::prepare(body, &invariants)?;
            rewrite::rewrite_body(body, &contexts);

            escapes::validate(body, &contexts, &invariants)
        })
    }

    pub fn validate_invariant_reference_contexts(&self) -> Result<(), MirLowerError> {
        let invariants = invariant_registry(&self.items);
        let mut items = self.items.clone();

        visit_bodies(&mut items, &mut |body| {
            let contexts = contexts::prepare(body, &invariants)?;

            escapes::validate(body, &contexts, &invariants)
        })
    }
}

fn invariant_registry(items: &[GlobalItem]) -> Invariants {
    let mut registry = HashMap::new();

    for item in items {
        match item {
            GlobalItem::Struct(definition) => {
                if let Some(invariant) = &definition.invariant {
                    registry.insert(definition.name.clone(), invariant.clone());
                }
            }
            GlobalItem::Impl { methods, .. } => registry.extend(invariant_registry(methods)),
            _ => {}
        }
    }

    registry
}

fn visit_bodies(
    items: &mut [GlobalItem],
    visitor: &mut impl FnMut(&mut MirBody) -> Result<(), MirLowerError>,
) -> Result<(), MirLowerError> {
    for item in items {
        match item {
            GlobalItem::Function {
                body: Some(body), ..
            } => visitor(body)?,
            GlobalItem::Impl { methods, .. } => visit_bodies(methods, visitor)?,
            _ => {}
        }
    }

    Ok(())
}
