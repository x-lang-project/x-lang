mod calls;
use inkwell::types;
use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Operator;
use x_ast::Type;
use x_diagnostics::backend_messages;

#[derive(Clone, Copy)]
enum ValueKind {
    Integer,
    Pointer,
    Aggregate,
    Other,
}

#[derive(Clone, Copy)]
enum Coercion {
    Identity,
    IntegerCast { signed: bool },
    PointerCast,
    LoadAggregate,
    SpillAggregate,
}

#[derive(Clone, Copy)]
struct CallArgument<'a> {
    function: &'a str,
    index: usize,
}

impl ValueKind {
    fn of(value: BasicValueEnum<'_>) -> Self {
        Self::of_type(value.get_type())
    }

    fn of_type(value_type: types::BasicTypeEnum<'_>) -> Self {
        if value_type.is_int_type() {
            Self::Integer
        } else if value_type.is_pointer_type() {
            Self::Pointer
        } else if value_type.is_struct_type() || value_type.is_array_type() {
            Self::Aggregate
        } else {
            Self::Other
        }
    }
}

pub(super) struct PointerArithmeticTypes<'a> {
    pub pointee: Option<&'a Type>,
    pub offset: &'a Type,
}

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn emit_pointer_arithmetic(
        &self,
        operator: &Operator,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
        types: PointerArithmeticTypes<'_>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenerationError> {
        let PointerArithmeticTypes {
            pointee: pointee_type,
            offset: offset_type,
        } = types;

        if !matches!(operator, Operator::Add | Operator::Subtract) {
            return Ok(None);
        }

        let (pointer, mut offset) = match (left.is_pointer_value(), right.is_pointer_value()) {
            (true, false) if right.is_int_value() => {
                (left.into_pointer_value(), right.into_int_value())
            }
            (false, true) if left.is_int_value() && matches!(operator, Operator::Add) => {
                (right.into_pointer_value(), left.into_int_value())
            }
            _ => return Ok(None),
        };

        if !offset_type.is_integer() {
            return Err(CodeGenerationError::UnsupportedType(offset_type.clone()));
        }

        if offset.get_type() != self.usize_type() {
            offset = self.integer_cast_signed(
                offset,
                self.usize_type(),
                super::numeric::is_signed_integer_type(offset_type),
                "ptr_offset_widen",
            )?;
        }

        if matches!(operator, Operator::Subtract) {
            offset = self.integer_neg(offset, "ptr_neg_offset")?;
        }

        let pointee_type =
            pointee_type.ok_or(CodeGenerationError::UnsupportedType(Type::Unknown))?;
        let element_type = self.codegen.resolve_llvm_type(pointee_type)?;
        let gep = self.gep(element_type, pointer, offset, "ptr_element_offset")?;

        Ok(Some(gep.into()))
    }

    pub(super) fn normalize_value_for_store(
        &self,
        source: super::numeric::TypedValue<'_, 'ctx>,
        expected_ast_type: &Type,
        expected_llvm_type: types::BasicTypeEnum<'ctx>,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let super::numeric::TypedValue {
            value: output_value,
            value_type: source_ast_type,
        } = source;

        if output_value.get_type() == expected_llvm_type {
            return self.emit_store_coercion(
                output_value,
                expected_llvm_type,
                Coercion::Identity,
                name,
            );
        }

        if matches!(expected_ast_type, Type::Unknown | Type::Infer(_))
            && (output_value.is_struct_value() || output_value.is_array_value())
        {
            return Ok(output_value);
        }

        if matches!(expected_ast_type, Type::Array(_)) && output_value.is_array_value() {
            return Ok(output_value);
        }

        let coercion = match (
            ValueKind::of(output_value),
            ValueKind::of_type(expected_llvm_type),
        ) {
            (ValueKind::Integer, ValueKind::Integer)
                if !matches!(
                    source_ast_type,
                    Type::Unknown | Type::Infer(_) | Type::TypeParameter(_)
                ) =>
            {
                Coercion::IntegerCast {
                    signed: super::numeric::is_signed_integer_type(source_ast_type),
                }
            }
            (ValueKind::Pointer, ValueKind::Pointer) => Coercion::PointerCast,
            (ValueKind::Pointer, ValueKind::Aggregate) => Coercion::LoadAggregate,
            (ValueKind::Aggregate, ValueKind::Pointer) => Coercion::SpillAggregate,
            _ => {
                return Err(CodeGenerationError::LlvmInternalError(backend_messages::cannot_store_llvm_value_value_into_mir_type_value_llvm_type_value(output_value.get_type(), expected_ast_type, expected_llvm_type)));
            }
        };

        self.emit_store_coercion(output_value, expected_llvm_type, coercion, name)
    }

    fn emit_store_coercion(
        &self,
        output_value: BasicValueEnum<'ctx>,
        expected_llvm_type: types::BasicTypeEnum<'ctx>,
        coercion: Coercion,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        match coercion {
            Coercion::Identity => Ok(output_value),
            Coercion::IntegerCast { signed } => {
                let source_integer = output_value.into_int_value();
                let destination_integer_type = expected_llvm_type.into_int_type();

                if source_integer.get_type().get_bit_width()
                    == destination_integer_type.get_bit_width()
                {
                    return Ok(source_integer.into());
                }

                self.integer_cast_signed(source_integer, destination_integer_type, signed, name)
                    .map(|value| value.into())
            }
            Coercion::PointerCast => self.bit_cast(
                output_value.into_pointer_value().into(),
                expected_llvm_type,
                name,
            ),
            Coercion::LoadAggregate => {
                let pointer = output_value.into_pointer_value();

                self.load(
                    expected_llvm_type,
                    pointer,
                    &format!("{name}_load_aggregate"),
                )
            }
            Coercion::SpillAggregate => {
                let storage = self.alloca(output_value.get_type(), name)?;
                self.store(storage, output_value)?;

                Ok(storage.into())
            }
        }
    }
}
