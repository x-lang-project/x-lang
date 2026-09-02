use super::FunctionCompiler;
use crate::function_helpers::is_pointer_like_ast_type;
use x_ast::Operator;
use x_ast::Type;
use x_mir::ResolvedCall;
use x_mir::{Operand, Place, Rvalue};

impl FunctionCompiler<'_, '_> {
    fn get_field_place_type(&self, base: &Place, field_name: &str, value_type: &Type) -> Type {
        if !matches!(value_type, Type::Unknown | Type::Infer(_)) {
            return value_type.clone();
        }

        let base_type = match self.get_place_type(base) {
            Type::Reference { inner, .. } | Type::RawPointer { inner, .. } => *inner,
            other => other,
        };

        match base_type {
            _ => value_type.clone(),
        }
    }

    fn get_index_place_type(&self, base: &Place, value_type: &Type) -> Type {
        let mut container = self.get_place_type(base);

        while let Type::Reference { inner, .. } = container {
            container = *inner;
        }

        match container {
            Type::String => Type::Character,
            Type::FixedSizedArray(inner, _) | Type::Array(inner) => *inner,
            Type::RawPointer { inner, .. } => *inner,
            _ => value_type.clone(),
        }
    }

    pub(super) fn get_place_type(&self, place: &Place) -> Type {
        match place {
            Place::Local(identifier) => self
                .frame
                .local_type(*identifier)
                .cloned()
                .or_else(|| {
                    self.body
                        .locals
                        .get(identifier)
                        .map(|left_value| left_value.value_type.clone())
                })
                .unwrap_or(Type::Unknown),
            Place::Dereference { value_type, .. } => value_type.clone(),
            Place::Global { value_type, .. } => value_type.clone(),
            Place::Index {
                base, value_type, ..
            } => self.get_index_place_type(base, value_type),
            Place::Field {
                base,
                field_name,
                value_type,
            } => self.get_field_place_type(base, field_name, value_type),
        }
    }
}
