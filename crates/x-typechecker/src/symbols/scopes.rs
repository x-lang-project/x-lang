use std::collections::HashMap;

use super::SymbolTable;
use x_ast::Type;

impl SymbolTable {
    pub fn enter_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if self.variables.len() > 1 {
            self.variables.pop();
        }
    }

    pub fn add_variable(&mut self, name: String, value_type: Type) {
        if self.variables.is_empty() {
            self.variables.push(HashMap::new());
        }

        if let Some(scope) = self.variables.last_mut() {
            scope.insert(name, value_type);
        }
    }

    pub fn add_global_constant(&mut self, name: String, value_type: Type) {
        self.global_constants.insert(name.clone());
        self.add_variable(name, value_type);
    }

    pub fn is_global_constant(&self, name: &str) -> bool {
        self.global_constants.contains(name)
    }

    pub fn get_variable(&self, name: &str) -> Option<&Type> {
        self.get_variable_with_scope(name)
            .map(|(value_type, _)| value_type)
    }

    pub fn resolve_variable(
        &self,
        name: &str,
        current_function: Option<&str>,
    ) -> Option<(String, Type)> {
        self.resolve_variable_with_scope(name, current_function)
            .map(|(name, value_type, _)| (name, value_type))
    }

    pub fn resolve_variable_with_scope(
        &self,
        name: &str,
        current_function: Option<&str>,
    ) -> Option<(String, Type, bool)> {
        if let Some((value_type, is_global)) = self.get_variable_with_scope(name) {
            return Some((name.to_string(), value_type.clone(), is_global));
        }

        if name.contains("::") {
            return None;
        }

        if let Some(current_function) = current_function
            && let Some(namespace) = x_ast::intrinsics::function_owner_name(current_function)
        {
            let candidate = format!("{namespace}::{name}");

            if let Some((value_type, is_global)) = self.get_variable_with_scope(&candidate) {
                return Some((candidate, value_type.clone(), is_global));
            }
        }

        let mut matches = self
            .variables
            .iter()
            .enumerate()
            .rev()
            .flat_map(|(index, scope)| {
                scope
                    .iter()
                    .filter(move |(candidate, _)| {
                        x_ast::intrinsics::function_leaf_name(candidate) == name
                    })
                    .map(move |(candidate, value_type)| (candidate, value_type, index == 0))
            });

        let first = matches.next()?;

        if matches.next().is_none() {
            Some((first.0.clone(), first.1.clone(), first.2))
        } else {
            None
        }
    }

    fn get_variable_with_scope(&self, name: &str) -> Option<(&Type, bool)> {
        self.variables
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, scope)| scope.get(name).map(|value_type| (value_type, index == 0)))
    }
}
