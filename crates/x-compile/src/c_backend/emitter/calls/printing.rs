use super::*;

impl Emitter<'_> {
    pub(super) fn print_call(
        &self,
        second_value: &MirBody,
        kind: BuiltinKind,
        arguments: &[Operand],
    ) -> Result<Value> {
        let [argument] = arguments else {
            return Err(backend_messages::c_emission_builtin_arity().into());
        };

        let value = self.operand(second_value, argument, None)?;
        let newline = kind == BuiltinKind::Println;

        if !matches!(kind, BuiltinKind::Print | BuiltinKind::Println) {
            return Err(backend_messages::c_emission_builtin(kind).into());
        }

        if value.value_type == Type::String {
            let write = format!(
                "x_runtime_write_stdout((const char*)({})->data,({})->length)",
                value.code, value.code,
            );

            return Ok(Value {
                code: if newline {
                    format!("({write},printf(\"\\n\"))")
                } else {
                    write
                },
                value_type: Type::Void,
            });
        }

        let fmt = match value.value_type {
            Type::Character => "%c",
            Type::Boolean => "%s",
            Type::F32 | Type::F64 => "%g",
            Type::I64 | Type::ISize => "%lld",
            Type::U64 | Type::USize => "%llu",
            Type::I8 | Type::I16 | Type::I32 => "%d",
            Type::U8 | Type::U16 | Type::U32 => "%u",
            _ => {
                return Err(
                    backend_messages::c_emission_unsupported_type(&value.value_type).into(),
                );
            }
        };
        let fmt = if newline {
            format!("{fmt}\n")
        } else {
            fmt.to_owned()
        };
        let code = match value.value_type {
            Type::Boolean => format!("(({})?\"true\":\"false\")", value.code),
            Type::F32 | Type::F64 => format!("((double)({}))", value.code),
            Type::I8 | Type::I16 | Type::I32 => {
                let expected = Type::I32;
                let element = self.operand(second_value, argument, Some(&expected))?;

                format!("((int)({}))", element.code)
            }
            Type::U8 | Type::U16 | Type::U32 => format!("((unsigned int)({}))", value.code),
            Type::I64 | Type::ISize => signed_view(&value)?,
            Type::Character => format!("((int)({}))", value.code),
            _ => value.code,
        };

        Ok(Value {
            code: format!("printf({}, {code})", string(&fmt)),
            value_type: Type::Void,
        })
    }
}
