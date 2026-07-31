use super::ExpressionLowerer;
use crate::errors::MirLowerError;
use crate::{Operand, Place};
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::backend_messages;

impl ExpressionLowerer<'_> {
    pub(super) fn resolve_identifier_place(&self, name: String) -> Result<Place, MirLowerError> {
        if let Some(local_identifier) = self.variable_mappings.get(&name).copied() {
            Ok(Place::Local(local_identifier))
        } else if let Some(global_type) = self.global_mappings.get(&name).cloned() {
            Ok(Place::Global {
                name,
                value_type: global_type,
            })
        } else if let Some((global_name, global_type)) = self.resolve_unique_global_suffix(&name) {
            Ok(Place::Global {
                name: global_name,
                value_type: global_type,
            })
        } else {
            Err(MirLowerError::UnallocatedVariable(name))
        }
    }

    pub(super) fn resolve_unique_global_suffix(&self, name: &str) -> Option<(String, Type)> {
        let suffix = format!("::{name}");
        let mut matches = self
            .global_mappings
            .iter()
            .filter(|(candidate, _)| candidate.ends_with(&suffix));
        let (candidate, value_type) = matches.next()?;

        matches
            .next()
            .is_none()
            .then(|| (candidate.clone(), value_type.clone()))
    }
}
