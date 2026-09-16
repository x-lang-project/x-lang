use super::FunctionCompiler;
use crate::function_helpers::is_pointer_like_ast_type;
use x_ast::Operator;
use x_ast::Type;
use x_mir::ResolvedCall;
use x_mir::{Operand, Place, Rvalue};

impl FunctionCompiler<'_, '_> {
    fn infer_binary_value_type(
        &self,
        operator: &Operator,
        left: &Operand,
        right: &Operand,
    ) -> Option<Type> {
        if matches!(operator, Operator::Add | Operator::Subtract)
            && let Some(pointer_type) = self.pointer_operand_type(left).or_else(|| {
                if matches!(operator, Operator::Add) {
                    self.pointer_operand_type(right)
                } else {
                    None
                }
            })
        {
            return Some(pointer_type);
        }

        if matches!(
            operator,
            Operator::Equal
                | Operator::NotEqual
                | Operator::LessThan
                | Operator::LessThanOrEqual
                | Operator::GreaterThan
                | Operator::GreaterThanOrEqual
                | Operator::And
                | Operator::Or
        ) {
            return Some(Type::Boolean);
        }

        let left_type = self.operand_ast_type(left);
        let right_type = self.operand_ast_type(right);

        if matches!(left_type, Some(Type::F64)) || matches!(right_type, Some(Type::F64)) {
            return Some(Type::F64);
        }

        if matches!(left_type, Some(Type::F32)) || matches!(right_type, Some(Type::F32)) {
            return Some(Type::F32);
        }

        left_type
            .filter(|value_type| !matches!(value_type, Type::Unknown | Type::Infer(_)))
            .or_else(|| {
                right_type
                    .filter(|value_type| !matches!(value_type, Type::Unknown | Type::Infer(_)))
            })
    }

    fn infer_place_value_type(&self, place: &Place) -> Option<Type> {
        if let Place::Dereference { base, value_type } = place
            && matches!(value_type, Type::Unknown | Type::Infer(_))
        {
            let pointee_type = match self.get_place_type(base) {
                Type::RawPointer { inner, .. } | Type::Reference { inner, .. } => *inner,
                _ => return None,
            };

            return Some(pointee_type);
        }

        let value_type = self.get_place_type(place);

        (!matches!(value_type, Type::Infer(_) | Type::Unknown)).then_some(value_type)
    }

    fn get_field_place_type(&self, base: &Place, field_name: &str, value_type: &Type) -> Type {
        if !matches!(value_type, Type::Unknown | Type::Infer(_)) {
            return value_type.clone();
        }

        let base_type = match self.get_place_type(base) {
            Type::Reference { inner, .. } | Type::RawPointer { inner, .. } => *inner,
            other => other,
        };

        match base_type {
            Type::Custom(struct_name)
            | Type::GenericInstance {
                name: struct_name, ..
            } => self
                .codegen
                .resolve_struct_field_type(&struct_name, field_name)
                .unwrap_or_else(|| value_type.clone()),
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

    pub(super) fn infer_call_result_type(
        &self,
        function: &str,
        arguments: &[Operand],
        expected_type: &Type,
    ) -> Option<Type> {
        if !matches!(expected_type, Type::Unknown | Type::Infer(_)) {
            return Some(expected_type.clone());
        }

        match self.body.call_targets.get(function)? {
            ResolvedCall::Builtin(_) => self
                .body
                .builtin_call(function, arguments, None)?
                .return_type
                .clone(),
            ResolvedCall::Source { return_type, .. } => return_type.clone(),
        }
        .filter(|value_type| !matches!(value_type, Type::Unknown | Type::Infer(_) | Type::Void))
    }

    pub(super) fn infer_rvalue_type(&self, rvalue: &Rvalue) -> Option<Type> {
        match rvalue {
            Rvalue::Use(Operand::Copy(place) | Operand::Move(place)) => {
                self.infer_place_value_type(place)
            }
            Rvalue::Use(operand) => self.operand_ast_type(operand),
            Rvalue::Call {
                function,
                arguments,
            }
            | Rvalue::TryCall {
                function,
                arguments,
            }
            | Rvalue::CatchCall {
                function,
                arguments,
                ..
            } => self.infer_call_result_type(function, arguments, &Type::Unknown),
            Rvalue::BinaryOperator(operator, left, right) => {
                self.infer_binary_value_type(operator, left, right)
            }
            Rvalue::PointerOffset { base, .. } => self.pointer_operand_type(base),
            _ => None,
        }
    }

    fn pointer_operand_type(&self, operand: &Operand) -> Option<Type> {
        self.operand_ast_type(operand)
            .filter(is_pointer_like_ast_type)
    }
}
