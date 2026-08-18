use crate::TypeChecker;
use x_ast::{Expression, ExpressionKind, Layout, Type};
use x_diagnostics::frontend_messages::ExecutableTypeReason;

impl TypeChecker {
    fn is_soa_struct_type(&self, value_type: &Type) -> bool {
        let name = match value_type {
            Type::Custom(name) | Type::GenericInstance { name, .. } => name,
            _ => return false,
        };

        self.symbols.records.layout(name) == Some(&Layout::SoA)
    }
}
