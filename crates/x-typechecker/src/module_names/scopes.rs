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

    pub(super) fn body(&mut self, statements: &mut [Statement]) {
        self.value_body(statements, None, None);
    }

    pub(super) fn generics(&mut self, generics: &mut [GenericParameter]) {
        if let Some(scope) = self.scopes.last_mut() {
            for parameter in generics.iter() {
                match parameter {
                    _ => {}
                }
            }
        }

        for parameter in generics {
            match parameter {
                _ => {}
            }
        }
    }

    pub(super) fn implementation(&mut self, definition: &mut ImplDefinition) {
        self.scopes.push(Scope {
            types: HashSet::from(["Self".into()]),
            ..Scope::default()
        });
        self.generics(&mut definition.generic_parameters);
        if !self.type_is_local(&definition.trait_name) {
            self.qualify(&mut definition.trait_name, self.types);
        }

        self.visit_type(&mut definition.type_name);
        for value_type in definition.associated_types.values_mut() {
            self.visit_type(value_type);
        }

        for method in &mut definition.methods {
            self.visit_statement(method);
        }

        self.scopes.pop();
    }

    pub(super) fn function(
        &mut self,
        generic_parameters: &mut [GenericParameter],
        parameters: &mut [Parameter],
        return_type: &mut Type,
        body: Option<&mut Vec<Statement>>,
    ) {
        self.scopes.push(Scope::default());
        self.generics(generic_parameters);
        for parameter in parameters {
            self.visit_type(&mut parameter.value_type);

            if let Some(scope) = self.scopes.last_mut() {
                scope.values.insert(parameter.name.clone());
            }
        }

        self.visit_type(return_type);
        if let Some(body) = body {
            self.body(body);
        }

        self.scopes.pop();
    }

    pub(super) fn value_body(
        &mut self,
        statements: &mut [Statement],
        expression: Option<&mut Expression>,
        pattern: Option<&Pattern>,
    ) {
        self.scopes.push(Scope::default());
        if let Some(pattern) = pattern {
            self.bind_pattern(pattern);
        }

        self.bind_declarations(statements);
        for statement in statements {
            self.visit_statement(statement);
        }

        if let Some(expression) = expression {
            self.visit_expression(expression);
        }

        self.scopes.pop();
    }

    pub(super) fn qualify(&self, name: &mut String, symbols: &HashSet<String>) {
        if !name.contains("::") && symbols.contains(name) {
            *name = format!("{}::{name}", self.prefix);
        }
    }
}
