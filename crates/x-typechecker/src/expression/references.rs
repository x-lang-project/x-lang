use crate::{TypeChecker, errors::TypeError};
use x_ast::{Expression, ExpressionKind, Layout, Type};
use x_diagnostics::frontend_messages::ExecutableTypeReason;

impl TypeChecker {
    fn check_soa_whole_address(&self, target: &Expression) -> Result<(), TypeError> {
        if matches!(target.kind, ExpressionKind::ArrayAccess { .. })
            && self.is_soa_struct_type(&target.value_type)
        {
            return Err(TypeError::ExecutableType {
                value_type: target.value_type.clone(),
                reason: ExecutableTypeReason::SoaWholeValueAddress,
            });
        }

        Ok(())
    }

    fn is_soa_struct_type(&self, value_type: &Type) -> bool {
        let name = match value_type {
            Type::Custom(name) | Type::GenericInstance { name, .. } => name,
            _ => return false,
        };

        self.symbols.records.layout(name) == Some(&Layout::SoA)
    }
}
