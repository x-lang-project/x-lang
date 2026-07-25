use super::MirLowerer;
use crate::errors::MirLowerError;

impl MirLowerer {
    pub(super) fn with_variable_scope<Value>(
        &mut self,
        lower: impl FnOnce(&mut Self) -> Result<Value, MirLowerError>,
    ) -> Result<Value, MirLowerError> {
        let mappings = self.variable_mappings.clone();
        let result = lower(self);
        self.variable_mappings = mappings;

        result
    }
}
