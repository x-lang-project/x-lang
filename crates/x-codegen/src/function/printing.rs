use std::slice;

use inkwell::values;
use inkwell::values::BasicMetadataValueEnum;

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::Operand;

#[derive(Clone, Copy)]
enum ScalarPrintFormat {
    Float,
    Signed32,
    Unsigned32,
    Signed64,
    Unsigned64,
}

impl ScalarPrintFormat {
    fn for_type(value_type: &Type) -> Option<Self> {
        Some(match value_type {
            Type::F32 | Type::F64 => Self::Float,
            Type::I8 | Type::I16 | Type::I32 => Self::Signed32,
            Type::U8 | Type::U16 | Type::U32 => Self::Unsigned32,
            Type::I64 | Type::ISize => Self::Signed64,
            Type::U64 | Type::USize => Self::Unsigned64,
            _ => return None,
        })
    }

    fn load_type(self) -> Type {
        match self {
            Self::Float => Type::F64,
            Self::Signed32 => Type::I32,
            Self::Unsigned32 => Type::U32,
            Self::Signed64 => Type::I64,
            Self::Unsigned64 => Type::U64,
        }
    }

    const fn format(self, newline: bool) -> &'static str {
        match (self, newline) {
            (Self::Float, false) => "%g",
            (Self::Float, true) => "%g\n",
            (Self::Signed32, false) => "%d",
            (Self::Signed32, true) => "%d\n",
            (Self::Unsigned32, false) => "%u",
            (Self::Unsigned32, true) => "%u\n",
            (Self::Signed64, false) => "%lld",
            (Self::Signed64, true) => "%lld\n",
            (Self::Unsigned64, false) => "%llu",
            (Self::Unsigned64, true) => "%llu\n",
        }
    }
}

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn emit_print_char(&self, arguments: &[Operand]) -> Result<(), CodeGenerationError> {
        let Some(argument) = arguments.first() else {
            return Err(CodeGenerationError::LlvmInternalError(
                backend_messages::io_print_char_expects_one_character_argument(),
            ));
        };

        let putchar = self.required_function("putchar")?;

        let value = self.load_operand_value_with_type(argument, &Type::Character)?;
        let integer_value = value.into_int_value();
        let i32_value = if integer_value.get_type().get_bit_width() == 32 {
            integer_value
        } else {
            self.integer_z_extend(integer_value, self.i32(), "print_char_i32")?
        };

        self.call(putchar, &[i32_value.into()], "print_char")?;

        Ok(())
    }

    pub(super) fn emit_print(&self, arguments: &[Operand]) -> Result<(), CodeGenerationError> {
        let Some(argument) = arguments.first() else {
            return Err(CodeGenerationError::LlvmInternalError(
                backend_messages::print_expects_one_argument(),
            ));
        };

        self.emit_print_value(argument, false)
    }

    pub(super) fn emit_println(&self, arguments: &[Operand]) -> Result<(), CodeGenerationError> {
        let [argument] = arguments else {
            return Err(CodeGenerationError::LlvmInternalError(
                backend_messages::println_expects_exactly_one_argument(),
            ));
        };

        self.emit_print_value(argument, true)
    }

    fn emit_print_value(
        &self,
        argument: &Operand,
        newline: bool,
    ) -> Result<(), CodeGenerationError> {
        match self.operand_ast_type(argument).unwrap_or(Type::Unknown) {
            Type::Character => {
                self.emit_print_char(slice::from_ref(argument))?;

                self.emit_newline_if_needed(newline)
            }
            Type::String => {
                self.emit_print_str(slice::from_ref(argument))?;

                self.emit_newline_if_needed(newline)
            }
            Type::Boolean => self.emit_print_boolean(argument, newline),
            value_type => match ScalarPrintFormat::for_type(&value_type) {
                Some(format) => self.emit_printf(argument, format, newline),
                None => Err(CodeGenerationError::LlvmInternalError(
                    backend_messages::print_does_not_support_value_of_type_type(value_type),
                )),
            },
        }
    }

    fn emit_print_boolean(
        &self,
        argument: &Operand,
        newline: bool,
    ) -> Result<(), CodeGenerationError> {
        let value = self
            .load_operand_value_with_type(argument, &Type::Boolean)?
            .into_int_value();

        let true_text = self.global_string_pointer("true", "print.bool.true")?;
        let false_text = self.global_string_pointer("false", "print.bool.false")?;
        let selected = self.select_pointer(
            value,
            true_text.as_pointer_value(),
            false_text.as_pointer_value(),
            "print.bool.text",
        )?;

        self.emit_print_cstr_pointer(selected)?;

        self.emit_newline_if_needed(newline)
    }

    fn emit_newline_if_needed(&self, newline: bool) -> Result<(), CodeGenerationError> {
        if newline {
            self.emit_putchar_newline()
        } else {
            Ok(())
        }
    }

    fn emit_printf(
        &self,
        argument: &Operand,
        format: ScalarPrintFormat,
        newline: bool,
    ) -> Result<(), CodeGenerationError> {
        let printf = self.required_function("printf")?;
        let fmt_pointer = self.global_string_pointer(format.format(newline), "print.fmt")?;
        let actual_load_type = if matches!(self.operand_ast_type(argument), Some(Type::F32)) {
            Type::F32
        } else {
            format.load_type()
        };

        let mut value = self.load_operand_value_with_type(argument, &actual_load_type)?;

        if value.is_float_value() {
            let float_value = value.into_float_value();

            if float_value.get_type() == self.context().f32_type() {
                value = self
                    .float_ext(float_value, self.context().f64_type(), "print.f64")?
                    .into();
            }
        }

        self.call(
            printf,
            &[
                fmt_pointer.as_pointer_value().into(),
                BasicMetadataValueEnum::from(value),
            ],
            "print.printf",
        )?;

        Ok(())
    }

    fn emit_putchar_newline(&self) -> Result<(), CodeGenerationError> {
        let putchar = self.required_function("putchar")?;

        self.call(
            putchar,
            &[self.i32().const_int('\n' as u64, false).into()],
            "println.newline",
        )?;

        Ok(())
    }

    pub(super) fn emit_print_str(&self, arguments: &[Operand]) -> Result<(), CodeGenerationError> {
        let Some(argument) = arguments.first() else {
            return Err(CodeGenerationError::LlvmInternalError(
                backend_messages::io_print_str_expects_one_string_argument(),
            ));
        };

        let value = self.load_operand_value_with_type(argument, &Type::String)?;
        let descriptor = value.into_pointer_value();
        let (data, length) = self.load_slice_descriptor(descriptor)?;
        let write = self.required_function(x_runtime::abi::WRITE_STDOUT)?;
        self.call(write, &[data.into(), length.into()], "print.str.bytes")?;

        Ok(())
    }

    fn emit_print_cstr_pointer(
        &self,
        str_pointer: values::PointerValue<'ctx>,
    ) -> Result<(), CodeGenerationError> {
        let printf = self.required_function("printf")?;
        let format = self.global_string_pointer("%s", "print.str.format")?;

        self.call(
            printf,
            &[format.as_pointer_value().into(), str_pointer.into()],
            "print.str",
        )?;

        Ok(())
    }
}
