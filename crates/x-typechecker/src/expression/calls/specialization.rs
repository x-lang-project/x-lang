use super::*;

impl TypeChecker {
    fn enqueue_specialized_body(
        &mut self,
        generic_name: &str,
        concrete_name: &str,
        function: &FunctionDefinition,
        substitutions: &HashMap<String, Type>,
    ) {
        let mut parameters = function.parameters.clone();

        for parameter in &mut parameters {
            substitute_type_in_parameter(parameter, self, substitutions);
        }

        let return_type = self.substitute_type(&function.return_type, substitutions);
        let mut body = function.body.clone();

        for statement in &mut body {
            substitute_types_in_statement(statement, self, substitutions);
        }

        let concrete_statement = Statement::Function {
            name: concrete_name.to_string(),
            generic_parameters: Vec::new(),
            parameters,
            return_type,
            body: Some(Box::new(body)),
            is_pure: function.is_pure,
            is_memoised: function.is_memoised,
            is_static: function.is_static,
            is_multi: function.is_multi,
            is_throws: function.is_throws,
            is_exported: function.is_exported,
        };

        let template_key = DeclarationKey::Function(
            generic_name.to_string(),
            function
                .parameters
                .iter()
                .map(|parameter| parameter.value_type.clone())
                .collect(),
        );

        if let (Some(origin), Some(key)) = (
            self.declaration_sources.get(&template_key).copied(),
            concrete_statement.declaration_key(),
        ) {
            self.declaration_sources.insert(key, origin);
        }

        self.pending_monomorphs.push(concrete_statement);
    }
}
