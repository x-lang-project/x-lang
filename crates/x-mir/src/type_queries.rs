use std::collections::HashMap;

use crate::{LocalDeclaration, LocalIdentifier, MirBody, Operand, Place};
use x_ast::Type;

impl MirBody {
    pub fn place_type(&self, place: &Place) -> Type {
        place_type(place, &self.locals).unwrap_or(Type::Unknown)
    }

    pub fn operand_type(&self, operand: &Operand) -> Type {
        operand_type(operand, &self.locals).unwrap_or(Type::Unknown)
    }
}

pub(crate) fn place_type(
    place: &Place,
    locals: &HashMap<LocalIdentifier, LocalDeclaration>,
) -> Option<Type> {
    match place {
        Place::Local(identifier) => locals.get(identifier).map(|local| local.value_type.clone()),
        Place::Dereference { value_type, .. }
        | Place::Global { value_type, .. }
        | Place::Field { value_type, .. }
        | Place::Index { value_type, .. } => Some(value_type.clone()),
    }
}

pub(crate) fn operand_type(
    operand: &Operand,
    locals: &HashMap<LocalIdentifier, LocalDeclaration>,
) -> Option<Type> {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => place_type(place, locals),
        Operand::ConstantInteger(value) => Some(if i32::try_from(*value).is_ok() {
            Type::I32
        } else {
            Type::I64
        }),
        Operand::ConstantFloat(_) => Some(Type::F64),
        Operand::ConstantBoolean(_) => Some(Type::Boolean),
        Operand::ConstantCharacter(_) => Some(Type::Character),
        Operand::ConstantString(_) => Some(Type::String),
        Operand::TypeLiteral(value_type) => Some(value_type.clone()),
    }
}
