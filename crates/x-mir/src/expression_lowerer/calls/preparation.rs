use std::collections::HashSet;

use crate::{Operand, Rvalue};
use x_ast::{Expression, ExpressionKind, Type, intrinsics};

pub(crate) struct PreparedCall {
    name: String,
    pub(crate) result_type: Type,
    symbolic_field_names: bool,
}

impl PreparedCall {
    pub(crate) fn new(name: String, result_type: Type, source_functions: &HashSet<String>) -> Self {
        let symbolic_field_names = !source_functions.contains(&name)
            && intrinsics::BuiltinCall::resolve(&name, &[], Some(&result_type)).is_some_and(
                |call| {
                    call.kind
                        == intrinsics::BuiltinKind::Intrinsic(intrinsics::Intrinsic::Layout(
                            intrinsics::LayoutIntrinsic::OffsetOf,
                        ))
                },
            );

        Self {
            name,
            result_type,
            symbolic_field_names,
        }
    }

    pub(crate) fn symbolic_argument(&self, argument: &Expression) -> Option<Operand> {
        match &argument.kind {
            ExpressionKind::Identifier(field) if self.symbolic_field_names => {
                Some(Operand::ConstantString(field.clone()))
            }
            _ => None,
        }
    }

    pub(crate) fn into_rvalue(self, arguments: Vec<Operand>) -> Rvalue {
        Rvalue::Call {
            function: self.name,
            arguments,
        }
    }
}
