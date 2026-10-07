use super::*;

impl Emitter<'_> {
    pub(super) fn binary(
        &mut self,
        second_value: &MirBody,
        input: (&Operator, &Operand, &Operand),
    ) -> Result<Value> {
        let (operator, left_value, current_result) = input;
        let lt = second_value.operand_type(left_value);
        let rt = second_value.operand_type(current_result);
        let mut first_value = self.operand(
            second_value,
            left_value,
            if matches!(left_value, Operand::ConstantInteger(_)) && width(&rt).is_some() {
                Some(&rt)
            } else {
                None
            },
        )?;
        let mut last_value = self.operand(
            second_value,
            current_result,
            if matches!(current_result, Operand::ConstantInteger(_)) && width(&lt).is_some() {
                Some(&lt)
            } else {
                None
            },
        )?;
        let cmp = matches!(
            operator,
            Operator::Equal
                | Operator::NotEqual
                | Operator::LessThan
                | Operator::LessThanOrEqual
                | Operator::GreaterThan
                | Operator::GreaterThanOrEqual
        );
        let token = match operator {
            Operator::Add => "+",
            Operator::Subtract => "-",
            Operator::Multiply => "*",
            Operator::Equal => "==",
            Operator::NotEqual => "!=",
            Operator::LessThan => "<",
            Operator::LessThanOrEqual => "<=",
            Operator::GreaterThan => ">",
            Operator::GreaterThanOrEqual => ">=",
            Operator::BitAnd | Operator::And => "&",
            Operator::BitOr | Operator::Or => "|",
            Operator::Xor => "^",
            _ => "",
        };
        let result = if let (Some(wa), Some(wz)) = (
            width(&first_value.value_type),
            width(&last_value.value_type),
        ) {
            let writer = wa.max(wz);
            let common = match writer {
                1 => Type::Boolean,
                8 => Type::U8,
                16 => Type::U16,
                32 => Type::U32,
                _ => Type::U64,
            };

            let is_signed = signed(&first_value.value_type);
            let sa = signed(&first_value.value_type);
            let sz = signed(&last_value.value_type);
            first_value = convert(first_value, &common, sa)?;
            last_value = convert(last_value, &common, sz)?;
            let code = if cmp {
                let ac = if is_signed {
                    signed_view(&first_value)?
                } else {
                    first_value.code
                };
                let zc = if is_signed {
                    signed_view(&last_value)?
                } else {
                    last_value.code
                };

                format!("(({ac}){token}({zc}))")
            } else if matches!(
                operator,
                Operator::Divide | Operator::Modulo | Operator::ShiftLeft | Operator::ShiftRight
            ) {
                let name = format!(
                    "_x_{}_{}_{}",
                    match operator {
                        Operator::Divide => "div",
                        Operator::Modulo => "rem",
                        Operator::ShiftLeft => "shl",
                        _ => "shr",
                    },
                    if is_signed { "s" } else { "u" },
                    writer
                );
                self.helpers.insert(name.clone());

                format!("{name}({}, {})", first_value.code, last_value.code)
            } else if writer >= 32 {
                format!("(({}){token}({}))", first_value.code, last_value.code)
            } else {
                bits(
                    &format!(
                        "((unsigned long long)({})){token}((unsigned long long)({}))",
                        first_value.code, last_value.code
                    ),
                    writer,
                )
            };

            Value {
                code,
                value_type: if cmp {
                    Type::Boolean
                } else {
                    if is_signed {
                        match writer {
                            8 => Type::I8,
                            16 => Type::I16,
                            32 => Type::I32,
                            64 => Type::I64,
                            _ => common,
                        }
                    } else {
                        common
                    }
                },
            }
        } else {
            if first_value.value_type != last_value.value_type
                || !matches!(first_value.value_type, Type::F32 | Type::F64)
            {
                return Err(backend_messages::c_emission_mixed_floating_pointer_operands().into());
            }

            if *operator == Operator::Modulo {
                return Ok(Value {
                    code: format!(
                        "{}({}, {})",
                        if first_value.value_type == Type::F32 {
                            "fmodf"
                        } else {
                            "fmod"
                        },
                        first_value.code,
                        last_value.code
                    ),
                    value_type: first_value.value_type,
                });
            }

            let token = if *operator == Operator::Divide {
                "/"
            } else {
                token
            };

            if token.is_empty() {
                return Err(backend_messages::c_emission_float_operator().into());
            }

            let code = format!("(({}){token}({}))", first_value.code, last_value.code);

            Value {
                code,
                value_type: if cmp {
                    Type::Boolean
                } else {
                    first_value.value_type
                },
            }
        };

        Ok(result)
    }
}

pub(super) fn helper(name: &str) -> Result<String> {
    let mut parts = name.rsplit('_');
    let writer: u32 = parts
        .next()
        .ok_or_else(|| EmitError::internal(backend_messages::c_emission_helper_width_is_missing()))?
        .parse()
        .map_err(|_| EmitError::internal(backend_messages::c_emission_helper_width_is_invalid()))?;
    let signed = parts.next().ok_or_else(|| {
        EmitError::internal(backend_messages::c_emission_helper_signedness_is_missing())
    })? == "s";
    let operator = parts.next().ok_or_else(|| {
        EmitError::internal(backend_messages::c_emission_helper_operation_is_missing())
    })?;
    let u = uint(writer);
    let text = sint(writer);
    let mask = if writer == 64 {
        "18446744073709551615ULL".to_owned()
    } else {
        format!("{}ULL", (1u64 << writer) - 1)
    };
    let body = match operator {
        "div" | "rem" => format!(
            "if(!b{} )_x_trap();return ({u})({}a{}{}b);",
            if signed {
                format!("||(a=={}ULL&&b=={mask})", 1u64 << (writer - 1))
            } else {
                String::new()
            },
            if signed {
                format!("({text})")
            } else {
                String::new()
            },
            if operator == "div" { "/" } else { "%" },
            if signed {
                format!("({text})")
            } else {
                String::new()
            }
        ),
        "shl" => format!("if(b>={writer})_x_trap();return ({u})((unsigned long long)a<<b);"),
        _ => format!(
            "if(b>={writer})_x_trap();return ({u})((a>>b){});",
            if signed {
                format!(
                    "|((b&&(a&{}ULL))?({mask}^({mask}>>b)):0)",
                    1u64 << (writer - 1)
                )
            } else {
                String::new()
            }
        ),
    };

    Ok(format!("static {u} {name}({u} a,{u} b){{{body}}}\n"))
}
