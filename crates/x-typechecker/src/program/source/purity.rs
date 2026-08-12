use x_ast::{ExpressionVisitor, Statement};
use x_diagnostics::frontend_messages::FrontendMessage;

struct PureBodies {
    missing: Option<String>,
}

impl ExpressionVisitor for PureBodies {
    fn visit_statement(&mut self, statement: &Statement) {
        if self.missing.is_some() {
            return;
        }

        match statement {
            Statement::Function {
                name,
                is_pure: true,
                body: None,
                ..
            } => self.missing = Some(name.clone()),
            Statement::Function {
                body: Some(body), ..
            } => self.visit_statements(body),
            Statement::ImplDeclaration(implementation) => {
                self.visit_statements(&implementation.methods);
            }
            Statement::TraitDeclaration(definition) => {
                for method in definition.methods.values().flatten() {
                    if matches!(method, Statement::Function { body: Some(_), .. }) {
                        self.visit_statement(method);
                    }
                }
            }
            _ => x_ast::walk_statement_expressions(self, statement),
        }
    }
}
