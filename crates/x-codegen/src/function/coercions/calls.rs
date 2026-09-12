use super::*;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(in crate::function) fn normalize_call_argument(
        &self,
        source: super::super::numeric::TypedValue<'_, 'ctx>,
        parameter: BasicValueEnum<'ctx>,
        callee: &str,
        index: usize,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let super::super::numeric::TypedValue {
            value: element,
            value_type: source_type,
        } = source;
        let argument = CallArgument {
            function: callee,
            index,
        };

        if element.get_type() == parameter.get_type() {
            return self.emit_call_coercion(element, parameter, Coercion::Identity, argument);
        }

        let coercion = match (
            ValueKind::of_type(element.get_type()),
            ValueKind::of_type(parameter.get_type()),
        ) {
            (ValueKind::Integer, ValueKind::Integer)
                if !matches!(
                    source_type,
                    Type::Unknown | Type::Infer(_) | Type::TypeParameter(_)
                ) =>
            {
                Coercion::IntegerCast {
                    signed: super::super::numeric::is_signed_integer_type(source_type),
                }
            }
            (ValueKind::Pointer, ValueKind::Pointer) => Coercion::PointerCast,
            (ValueKind::Pointer, ValueKind::Aggregate) => Coercion::LoadAggregate,
            (ValueKind::Aggregate, ValueKind::Pointer) => Coercion::SpillAggregate,
            _ => {
                return Err(CodeGenerationError::LlvmInternalError(backend_messages::cannot_pass_argument_value_to_value_llvm_type_mismatch_value_value(index, callee, element.get_type(), parameter.get_type())));
            }
        };

        self.emit_call_coercion(element, parameter, coercion, argument)
    }

    pub(super) fn emit_call_coercion(
        &self,
        element: BasicValueEnum<'ctx>,
        parameter: BasicValueEnum<'ctx>,
        coercion: Coercion,
        argument: CallArgument<'_>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let CallArgument {
            function: callee,
            index,
        } = argument;

        match coercion {
            Coercion::Identity => Ok(element),
            Coercion::IntegerCast { signed } => self
                .integer_cast_signed(
                    element.into_int_value(),
                    parameter.into_int_value().get_type(),
                    signed,
                    &format!("{callee}_arg{index}_intcast"),
                )
                .map(|value| value.into()),
            Coercion::PointerCast => self.emit_store_coercion(
                element,
                parameter.get_type(),
                coercion,
                &format!("{callee}_arg{index}_ptrcast"),
            ),
            Coercion::LoadAggregate => self.load(
                parameter.get_type(),
                element.into_pointer_value(),
                &format!("{callee}_arg{index}_load_aggregate"),
            ),
            Coercion::SpillAggregate => self.emit_store_coercion(
                element,
                parameter.get_type(),
                coercion,
                &format!("{callee}_arg{index}_spill"),
            ),
        }
    }
}
