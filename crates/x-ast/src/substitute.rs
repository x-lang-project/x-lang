use crate::TypeVisitorMut;
use crate::{Expression, Statement, Type};

struct SelfSubstituter<'a>(&'a Type);

impl TypeVisitorMut for SelfSubstituter<'_> {
    fn visit_type(&mut self, value_type: &mut Type) {
        substitute_self_in_type(value_type, self.0);
    }
}

pub fn substitute_self_in_statement(statement: &mut Statement, self_type: &Type) {
    TypeVisitorMut::visit_statement(&mut SelfSubstituter(self_type), statement);
}

pub fn substitute_self_in_expression(expression: &mut Expression, self_type: &Type) {
    TypeVisitorMut::visit_expression(&mut SelfSubstituter(self_type), expression);
}

pub fn substitute_self_in_type(value_type: &mut Type, self_type: &Type) {
    match value_type {
        Type::TypeParameter(name) | Type::Custom(name) if name == "Self" => {
            *value_type = self_type.clone();
        }
        _ => {
            for child in value_type.children_mut() {
                substitute_self_in_type(child, self_type);
            }
        }
    }
}
