use super::*;

impl ModuleQualifier<'_> {
    pub(super) fn type_is_local(&self, name: &str) -> bool {
        self.scopes
            .iter()
            .rev()
            .any(|scope| scope.types.contains(name))
    }

    pub(super) fn bind_pattern(&mut self, pattern: &Pattern) {
        let Some(scope) = self.scopes.last_mut() else {
            return;
        };

        match pattern {
            Pattern::Identifier(name) => {
                scope.values.insert(name.clone());
            }
            Pattern::Tuple(names)
            | Pattern::Enum {
                bindings: names, ..
            } => scope.values.extend(names.iter().cloned()),
            Pattern::Wildcard => {}
        }
    }

    pub(super) fn bind_declarations(&mut self, body: &[Statement]) {
        let Some(scope) = self.scopes.last_mut() else {
            return;
        };

        for statement in body {
            match statement {
                _ => {}
            }
        }
    }

    pub(super) fn qualify(&self, name: &mut String, symbols: &HashSet<String>) {
        if !name.contains("::") && symbols.contains(name) {
            *name = format!("{}::{name}", self.prefix);
        }
    }
}
