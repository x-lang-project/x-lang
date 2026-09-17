use inkwell::module::Linkage;
use inkwell::types::BasicType;
use inkwell::values::FunctionValue;

use crate::codegen::CodeGeneration;
use crate::codegen::items::FunctionDeclaration;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_mir::{FunctionMeta, LocalIdentifier, MirBody};

impl<'ctx> CodeGeneration<'ctx> {
    pub(in crate::codegen) fn signature_types(
        body: Option<&MirBody>,
        declared_parameters: &[Type],
        declared_return: &Type,
    ) -> (Vec<Type>, Type) {
        let return_value = body
            .and_then(|second_value| {
                second_value
                    .locals
                    .get(&second_value.return_local)
                    .map(|left_value| left_value.value_type.clone())
            })
            .unwrap_or_else(|| declared_return.clone());
        let parameters = body
            .map(Self::body_parameter_types)
            .unwrap_or_else(|| declared_parameters.to_vec());

        (parameters, return_value)
    }

    pub(in crate::codegen) fn body_parameter_types(body: &MirBody) -> Vec<Type> {
        (0..body.argument_count)
            .filter_map(|index| {
                body.locals
                    .get(&LocalIdentifier(index + 1))
                    .map(|local| local.value_type.clone())
            })
            .collect()
    }
}
