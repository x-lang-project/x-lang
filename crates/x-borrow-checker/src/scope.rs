mod lifecycle;

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use super::{BorrowCheckError, BorrowChecker, Region, VariableInfo};
use x_ast::Type;
use x_diagnostics::borrow_messages;

static EMPTY_VARIABLES: LazyLock<Arc<HashMap<String, VariableInfo>>> =
    LazyLock::new(|| Arc::new(HashMap::new()));
static EMPTY_SHADOWED: LazyLock<Arc<HashMap<String, Vec<VariableInfo>>>> =
    LazyLock::new(|| Arc::new(HashMap::new()));

#[derive(Clone)]
pub(super) struct Scope {
    pub(super) region: Region,
    pub(super) variables: Arc<HashMap<String, VariableInfo>>,
    pub(super) shadowed: Arc<HashMap<String, Vec<VariableInfo>>>,
    // Every mutable access marks the scope for loan expiry.
    pub(super) may_need_expiry: bool,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            region: Region::default(),
            variables: Arc::clone(&EMPTY_VARIABLES),
            shadowed: Arc::clone(&EMPTY_SHADOWED),
            may_need_expiry: false,
        }
    }
}

// Expiry hints are excluded from fixed-point equality.
impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        self.region == other.region
            && (Arc::ptr_eq(&self.variables, &other.variables) || self.variables == other.variables)
            && (Arc::ptr_eq(&self.shadowed, &other.shadowed) || self.shadowed == other.shadowed)
    }
}

impl Eq for Scope {}

impl Scope {
    pub(super) fn merge_branches(&mut self, then_scope: &Self, else_scope: &Self) {
        self.may_need_expiry |= then_scope.may_need_expiry || else_scope.may_need_expiry;
        if Arc::ptr_eq(&self.variables, &then_scope.variables)
            && Arc::ptr_eq(&self.variables, &else_scope.variables)
            && Arc::ptr_eq(&self.shadowed, &then_scope.shadowed)
            && Arc::ptr_eq(&self.shadowed, &else_scope.shadowed)
        {
            return;
        }

        for (name, generations) in Arc::make_mut(&mut self.shadowed) {
            for (generation, variable) in generations.iter_mut().enumerate() {
                if let (Some(then_variable), Some(else_variable)) = (
                    then_scope.storage(name, generation),
                    else_scope.storage(name, generation),
                ) {
                    variable.merge_branches(then_variable, else_variable);
                }
            }
        }

        for (name, variable) in Arc::make_mut(&mut self.variables).iter_mut() {
            let Some(then_variable) = then_scope.variables.get(name) else {
                continue;
            };

            let Some(else_variable) = else_scope.variables.get(name) else {
                continue;
            };

            variable.merge_branches(then_variable, else_variable);
        }
    }

    pub(super) fn insert(&mut self, name: String, value: VariableInfo) {
        self.may_need_expiry |= !value.borrows.is_empty()
            || !value.field_borrows.is_empty()
            || !value.references.targets.is_empty()
            || !value.references.fields.is_empty();

        if let Some(previous) = Arc::make_mut(&mut self.variables).insert(name.clone(), value) {
            Arc::make_mut(&mut self.shadowed)
                .entry(name)
                .or_default()
                .push(previous);
        }
    }

    pub(super) fn generation(&self, name: &str) -> usize {
        self.shadowed.get(name).map_or(0, Vec::len)
    }

    pub(super) fn storage(&self, name: &str, generation: usize) -> Option<&VariableInfo> {
        if generation == self.generation(name) {
            self.variables.get(name)
        } else {
            self.shadowed.get(name)?.get(generation)
        }
    }

    pub(super) fn storage_mut(
        &mut self,
        name: &str,
        generation: usize,
    ) -> Option<&mut VariableInfo> {
        self.may_need_expiry = true;

        if generation == self.generation(name) {
            self.variables.get(name)?;

            Arc::make_mut(&mut self.variables).get_mut(name)
        } else {
            self.shadowed.get(name)?.get(generation)?;

            Arc::make_mut(&mut self.shadowed)
                .get_mut(name)?
                .get_mut(generation)
        }
    }

    pub(super) fn values_mut(&mut self) -> impl Iterator<Item = &mut VariableInfo> {
        self.may_need_expiry = true;
        let variables = if self.variables.is_empty() {
            None
        } else {
            Some(Arc::make_mut(&mut self.variables))
        };
        let shadowed = if self.shadowed.is_empty() {
            None
        } else {
            Some(Arc::make_mut(&mut self.shadowed))
        };

        variables
            .into_iter()
            .flat_map(|variables| variables.values_mut())
            .chain(
                shadowed
                    .into_iter()
                    .flat_map(|shadowed| shadowed.values_mut().flatten()),
            )
    }
}
