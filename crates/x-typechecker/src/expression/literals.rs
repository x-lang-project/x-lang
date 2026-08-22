use crate::{TypeChecker, errors::TypeError};
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

pub(super) fn check_string_literal(literal: &StringLiteral) -> Result<Type, TypeError> {
    if literal
        .parts
        .iter()
        .any(|part| matches!(part, StringPart::Interpolation(_)))
    {
        return Err(TypeError::ExecutableType {
            value_type: Type::String,
            reason: ExecutableTypeReason::StringInterpolation,
        });
    }

    Ok(Type::String)
}

pub(super) fn check_character_literal(value: char) -> Result<Type, TypeError> {
    if u32::from(value) > u8::MAX.into() {
        return Err(crate::type_mismatch!(
            "byte character in U+0000..U+00FF",
            format!("character U+{:04X}", u32::from(value))
        ));
    }

    Ok(Type::Character)
}
