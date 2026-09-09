use crate::function_helpers::{push_unique_candidate, strip_explicit_type_arguments};
use x_ast::intrinsics::function_owner_name;

pub(super) struct CallName<'a> {
    pub full: &'a str,
    pub base: &'a str,
}

impl<'a> CallName<'a> {
    pub(super) fn parse(full: &'a str) -> Self {
        Self {
            full,
            base: strip_explicit_type_arguments(full),
        }
    }

    pub(super) fn resolution_candidates(&self, caller: &str) -> Vec<String> {
        let mut candidates = Vec::new();
        push_unique_candidate(&mut candidates, self.full.to_string());
        push_unique_candidate(&mut candidates, self.base.to_string());

        if !self.base.contains("::") {
            if let Some(module_path) = function_owner_name(caller) {
                push_unique_candidate(&mut candidates, format!("{module_path}::{}", self.full));
                push_unique_candidate(&mut candidates, format!("{module_path}::{}", self.base));
            }

            push_unique_candidate(&mut candidates, format!("intrinsic::{}", self.base));
        }

        candidates
    }
}
