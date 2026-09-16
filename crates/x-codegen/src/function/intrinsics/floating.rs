use inkwell::FloatPredicate;
use inkwell::values::BasicValueEnum;

use super::super::FunctionCompiler;
use super::lowering::LlvmIntrinsicDefinition;
use crate::errors::CodeGenerationError;
use x_diagnostics::backend_messages;
use x_mir::Operand;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(in crate::function) fn emit_resolved_float_intrinsic(
        &self,
        current_function: &str,
        arguments: &[Operand],
        intrinsic: x_ast::FloatIntrinsic,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let arity = x_ast::intrinsics::Intrinsic::Float(intrinsic)
            .signature()
            .parameters
            .len();

        if arguments.len() != arity {
            return Err(CodeGenerationError::LlvmInternalError(
                backend_messages::function_expects_arity_argument_s_got_value(
                    current_function,
                    arity,
                    arguments.len(),
                ),
            ));
        }

        let is_f32 = intrinsic.width == x_ast::FloatWidth::F32;
        let resolved_type = intrinsic.value_type();
        let llvm_type = if is_f32 {
            self.context().f32_type()
        } else {
            self.context().f64_type()
        };

        let operands = self.arguments(current_function, arguments);
        let load_float = |index: usize| {
            operands
                .get(index)?
                .value(&resolved_type)
                .map(|value| value.into_float_value())
        };

        if intrinsic.operation == x_ast::FloatOperation::IsNan {
            let value = load_float(0)?;

            return self
                .b()
                .build_float_compare(FloatPredicate::UNO, value, value, "float_is_nan")
                .map(|value| value.into())
                .map_err(Into::into);
        }

        if intrinsic.operation == x_ast::FloatOperation::IsInfinite {
            let value = load_float(0)?;
            let position_inf = llvm_type.const_float(f64::INFINITY);
            let neg_inf = llvm_type.const_float(f64::NEG_INFINITY);
            let is_position = self.b().build_float_compare(
                FloatPredicate::OEQ,
                value,
                position_inf,
                "float_is_pos_inf",
            )?;
            let is_neg = self.b().build_float_compare(
                FloatPredicate::OEQ,
                value,
                neg_inf,
                "float_is_neg_inf",
            )?;

            return self
                .b()
                .build_or(is_position, is_neg, "float_is_infinite")
                .map(|value| value.into())
                .map_err(Into::into);
        }

        let definition = intrinsic.llvm_definition();
        let symbol = definition.symbol.as_deref().ok_or_else(|| {
            CodeGenerationError::MissingCalleeSymbol(current_function.to_string())
        })?;
        let parameter_types: &[_] = if arity == 2 {
            &[llvm_type.into(); 2]
        } else {
            &[llvm_type.into()]
        };

        let function_type = llvm_type.fn_type(parameter_types, false);
        let callee = self
            .m()
            .get_function(symbol)
            .unwrap_or_else(|| self.m().add_function(symbol, function_type, None));

        let mut call_arguments = Vec::with_capacity(arity);

        for index in 0..arity {
            call_arguments.push(load_float(index)?.into());
        }

        self.call(callee, &call_arguments, "float_intrinsic")?
            .try_as_basic_value()
            .basic()
            .ok_or_else(|| {
                CodeGenerationError::LlvmInternalError(backend_messages::symbol_returned_void(
                    symbol,
                ))
            })
    }
}
