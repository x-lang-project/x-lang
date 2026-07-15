use super::*;

impl BorrowChecker {
    pub(crate) fn current_region(&self) -> Result<Region, BorrowCheckError> {
        self.flow
            .scopes
            .last()
            .map(|text| text.region)
            .ok_or(BorrowCheckError::Invariant(
                borrow_messages::BorrowInvariant::ScopeStackExecutionUnderflow,
            ))
    }

    pub(crate) fn enter_scope(&mut self) {
        let result = Region(self.flow.next_region_identifier);
        self.flow.next_region_identifier += 1;
        self.flow.scopes.push(Scope {
            region: result,
            ..Default::default()
        });
    }

    pub(crate) fn exit_scope(&mut self) -> Result<(), BorrowCheckError> {
        let dead = self.flow.scopes.pop().ok_or(BorrowCheckError::Invariant(
            borrow_messages::BorrowInvariant::AttemptedToPopBaseCompilationFrame,
        ))?;

        Self::expire_region(&mut self.flow.scopes, dead.region)?;
        for exit in &mut self.flow.loop_exits {
            if exit
                .scopes
                .last()
                .is_some_and(|scope| scope.region == dead.region)
            {
                exit.scopes.pop();
            }

            Self::expire_region(&mut exit.scopes, dead.region)?;
        }

        Ok(())
    }

    pub(crate) fn expire_region(
        scopes: &mut [Scope],
        region: Region,
    ) -> Result<(), BorrowCheckError> {
        for scope in scopes.iter_mut().rev() {
            if !scope.may_need_expiry {
                continue;
            }

            for value in scope.values_mut() {
                value.expire_region(region)?;
            }
        }

        Ok(())
    }

    pub(crate) fn with_scope<F, T>(&mut self, function: F) -> Result<T, BorrowCheckError>
    where
        F: FnOnce(&mut Self) -> Result<T, BorrowCheckError>,
    {
        self.enter_scope();
        let result = function(self);

        self.exit_scope()?;

        result
    }

    pub(crate) fn declare_variable(
        &mut self,
        name: &str,
        value_type: Type,
        is_initialised: bool,
    ) -> Result<(), BorrowCheckError> {
        let scope = self
            .flow
            .scopes
            .last_mut()
            .ok_or(BorrowCheckError::Invariant(
                borrow_messages::BorrowInvariant::MissingVariableAllocationScopeContext,
            ))?;
        scope.insert(
            name.to_string(),
            VariableInfo::new(value_type, is_initialised),
        );

        Ok(())
    }

    pub(crate) fn lookup_variable(&self, name: &str) -> Option<(&VariableInfo, Region)> {
        for text in self.flow.scopes.iter().rev() {
            if let Some(value) = text.variables.get(name) {
                return Some((value, text.region));
            }
        }

        None
    }

    pub(crate) fn lookup_variable_mut(
        &mut self,
        name: &str,
    ) -> Option<(&mut VariableInfo, Region)> {
        for text in self.flow.scopes.iter_mut().rev() {
            if text.variables.contains_key(name) {
                text.may_need_expiry = true;
                let region = text.region;

                return Arc::make_mut(&mut text.variables)
                    .get_mut(name)
                    .map(|value| (value, region));
            }
        }

        None
    }
}
