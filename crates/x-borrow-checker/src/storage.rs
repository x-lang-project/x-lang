use std::collections;

use std::collections::HashMap;

use super::{Borrow, BorrowCheckError, ReferenceValue, Region};
use x_ast::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VariableInfo {
    pub value_type: Type,
    pub is_initialised: bool,
    pub is_moved: bool,
    pub moved_fields: collections::HashSet<String>,
    pub borrows: Vec<Borrow>,
    pub field_borrows: HashMap<String, Vec<Borrow>>,
    pub references: ReferenceValue,
}

impl VariableInfo {
    pub fn new(value_type: Type, is_initialised: bool) -> Self {
        Self {
            value_type,
            is_initialised,
            is_moved: false,
            moved_fields: Default::default(),
            borrows: Vec::new(),
            field_borrows: HashMap::new(),
            references: ReferenceValue::default(),
        }
    }

    pub(super) fn merge_branches(&mut self, then_variable: &Self, else_variable: &Self) {
        self.is_initialised = then_variable.is_initialised && else_variable.is_initialised;
        self.is_moved = then_variable.is_moved || else_variable.is_moved;
        self.moved_fields = then_variable
            .moved_fields
            .union(&else_variable.moved_fields)
            .cloned()
            .collect();
        self.references = then_variable.references.join(&else_variable.references);
        self.borrows = union(&then_variable.borrows, &else_variable.borrows);
        self.field_borrows = then_variable.field_borrows.clone();

        for (field, loans) in &else_variable.field_borrows {
            let previous = self.field_borrows.entry(field.clone()).or_default();
            *previous = union(previous, loans);
        }
    }

    pub(super) fn check_available(&self, name: &str) -> Result<(), BorrowCheckError> {
        self.check_place_available(name, None)
    }

    pub(super) fn check_place_available(
        &self,
        name: &str,
        field: Option<&str>,
    ) -> Result<(), BorrowCheckError> {
        if self.is_moved {
            return Err(BorrowCheckError::UseAfterMove(name.to_string()));
        }

        if self
            .moved_fields
            .iter()
            .any(|moved| field.is_none_or(|field| field_paths_overlap(field, moved)))
        {
            return Err(BorrowCheckError::UseAfterMove(format!(
                "{name}{}",
                field.map(|field| format!(".{field}")).unwrap_or_default()
            )));
        }

        if !self.is_initialised {
            return Err(BorrowCheckError::UseBeforeInitialisation(name.to_string()));
        }

        Ok(())
    }

    pub(super) fn create_borrow(
        &mut self,
        name: &str,
        field: Option<&str>,
        region: Region,
        is_mut: bool,
    ) -> Result<(), BorrowCheckError> {
        self.check_place_available(name, field)?;

        match field {
            Some(field) => self.borrow_field(name, field, region, is_mut),
            None => self.borrow_whole(name, region, is_mut),
        }
    }

    pub(super) fn check_read(
        &self,
        name: &str,
        field: Option<&str>,
    ) -> Result<(), BorrowCheckError> {
        self.check_place_available(name, field)?;
        if self.borrows.iter().any(|loan| loan.is_mut)
            || self.field_borrows.iter().any(|(borrowed, loans)| {
                loans.iter().any(|loan| loan.is_mut)
                    && field.is_none_or(|field| field_paths_overlap(field, borrowed))
            })
        {
            return Err(BorrowCheckError::ImmutableBorrowViolation(name.to_string()));
        }

        Ok(())
    }

    fn borrow_field(
        &mut self,
        name: &str,
        field: &str,
        region: Region,
        is_mut: bool,
    ) -> Result<(), BorrowCheckError> {
        if is_mut && !self.borrows.is_empty() {
            return Err(BorrowCheckError::MutableBorrowViolation(name.to_string()));
        }

        if !is_mut && self.borrows.iter().any(|second_value| second_value.is_mut) {
            return Err(BorrowCheckError::ImmutableBorrowViolation(name.to_string()));
        }

        for (borrowed_field, loans) in &self.field_borrows {
            if field_paths_overlap(field, borrowed_field)
                && loans.iter().any(|loan| is_mut || loan.is_mut)
            {
                return Err(if is_mut {
                    BorrowCheckError::MutableBorrowViolation(format!("{name}.{field}"))
                } else {
                    BorrowCheckError::ImmutableBorrowViolation(format!("{name}.{field}"))
                });
            }
        }

        let fb = self.field_borrows.entry(field.to_string()).or_default();
        fb.push(Borrow {
            region,
            is_mut,
            is_reborrow: false,
        });

        Ok(())
    }

    fn borrow_whole(
        &mut self,
        name: &str,
        region: Region,
        is_mut: bool,
    ) -> Result<(), BorrowCheckError> {
        if is_mut {
            if !self.borrows.is_empty() || self.field_borrows.values().any(|fb| !fb.is_empty()) {
                return Err(BorrowCheckError::MutableBorrowViolation(name.to_string()));
            }
        } else if self.borrows.iter().any(|second_value| second_value.is_mut)
            || self
                .field_borrows
                .values()
                .any(|fb| fb.iter().any(|borrow| borrow.is_mut))
        {
            return Err(BorrowCheckError::ImmutableBorrowViolation(name.to_string()));
        }

        self.borrows.push(Borrow {
            region,
            is_mut,
            is_reborrow: false,
        });

        Ok(())
    }

    pub(super) fn expire_region(&mut self, region: Region) -> Result<(), BorrowCheckError> {
        self.borrows.retain(|borrow| borrow.region != region);
        self.field_borrows.retain(|_, borrows| {
            borrows.retain(|borrow| borrow.region != region);

            !borrows.is_empty()
        });

        if !self.is_moved
            && let Some(target) = self
                .references
                .find_target(|target| target.origin_region == region)
        {
            return Err(BorrowCheckError::LifetimeTooShort {
                target: target.target_variable.clone(),
            });
        }

        Ok(())
    }
}

pub(super) fn field_paths_overlap(first: &str, second: &str) -> bool {
    let descendant_of = |path: &str, parent: &str| {
        path.strip_prefix(parent)
            .is_some_and(|suffix| suffix.starts_with('.'))
    };

    first == second || descendant_of(first, second) || descendant_of(second, first)
}

fn union(first: &[Borrow], second: &[Borrow]) -> Vec<Borrow> {
    let mut result = first.to_vec();

    for value in second {
        if !result.contains(value) {
            result.push(value.clone());
        }
    }

    result
}
