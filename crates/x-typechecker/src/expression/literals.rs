use crate::TypeChecker;
use x_ast::{StringLiteral, StringPart, Type};
use x_diagnostics::frontend_messages::ExecutableTypeReason;

impl TypeChecker {
    pub(super) fn integer_literal_type(&self, annotation: &Type) -> Type {
        let declared = self.resolve(annotation);

        if declared.is_integer() {
            declared
        } else {
            Type::I32
        }
    }

    pub(super) fn float_literal_type(&self, annotation: &Type) -> Type {
        let declared = self.resolve(annotation);

        if matches!(declared, Type::F32 | Type::F64) {
            declared
        } else {
            Type::F64
        }
    }
}
