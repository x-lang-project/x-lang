use inkwell::module::Linkage;
use inkwell::types::BasicType;
use inkwell::values::FunctionValue;

use crate::codegen::items::FunctionDeclaration;
use crate::codegen::{CodeGeneration, is_exported_entry_symbol};
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

    pub(in crate::codegen) fn internalize_private_definition(
        &self,
        name: &str,
        body: Option<&MirBody>,
        meta: &FunctionMeta,
    ) {
        if self.options.export_global_symbols || meta.is_exported {
            return;
        }

        if body.is_none() || is_exported_entry_symbol(self.llvm_function_symbol(name)) {
            return;
        }

        if let Some(function) = self.module.get_function(self.llvm_function_symbol(name)) {
            function.set_linkage(Linkage::Internal);
        }
    }

    pub fn declare_function_signature(
        &self,
        declaration: FunctionDeclaration<'_>,
    ) -> Result<FunctionValue<'ctx>, CodeGenerationError> {
        let FunctionDeclaration {
            name,
            body,
            parameter_types: parameters,
            return_type: return_value,
            meta,
        } = declaration;

        let llvm_parameter_types = parameters
            .iter()
            .map(|value_type| self.resolve_llvm_type(value_type).map(Into::into))
            .collect::<Result<Vec<_>, _>>()?;

        let abi_return = if meta.is_throws {
            Type::Tuple(vec![Type::Boolean, return_value.clone(), Type::String])
        } else {
            return_value.clone()
        };

        let actual_function_type = if name == x_ast::ENTRY_POINT_NAME && abi_return == Type::Void {
            self.context
                .i32_type()
                .fn_type(&llvm_parameter_types, false)
        } else if abi_return == Type::Void {
            self.context
                .void_type()
                .fn_type(&llvm_parameter_types, false)
        } else {
            self.resolve_llvm_type(&abi_return)?
                .fn_type(&llvm_parameter_types, false)
        };

        let llvm_name = self.llvm_function_symbol(name);
        let function = self.module.get_function(llvm_name).unwrap_or_else(|| {
            self.module
                .add_function(llvm_name, actual_function_type, None)
        });

        if body.is_none() {
            self.apply_external_c_abi_attributes(function, parameters, return_value);
        }

        self.apply_function_semantic_attributes(function, parameters, meta);

        Ok(function)
    }
}
