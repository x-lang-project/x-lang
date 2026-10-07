use super::*;

impl Emitter<'_> {
    pub(super) fn emit_terminator(
        &mut self,
        name: &str,
        block: &BasicBlock,
        output: &mut String,
    ) -> Result<()> {
        let signature = self.symbols.get(name).ok_or_else(|| {
            EmitError::internal(backend_messages::c_emission_registered_function_is_missing())
        })?;
        let body = signature.body;
        let return_value = signature.return_value.clone();

        match block
            .terminator
            .as_ref()
            .ok_or_else(|| EmitError::internal(backend_messages::c_emission_missing_terminator()))?
        {
            MirTerminator::Return => {
                if body.is_memoised {
                    output.push_str(&memoise::store(self.symbols.get(name).ok_or_else(
                        || {
                            EmitError::internal(
                                backend_messages::c_emission_registered_function_is_missing(),
                            )
                        },
                    )?)?);
                }

                if body.is_throws {
                    writeln!(
                        output,
                        "return ({}){{1,{},0}};",
                        throws_ctype(&return_value),
                        if return_value == Type::Void {
                            "0".into()
                        } else {
                            format!("v{}", body.return_local.0)
                        }
                    )?;
                } else if return_value == Type::Void {
                    output.push_str(if name == "main" {
                        "return 0;\n"
                    } else {
                        "return;\n"
                    })
                } else {
                    writeln!(output, "return v{};", body.return_local.0)?;
                }
            }
            MirTerminator::Goto(identifier) => writeln!(output, "goto b{};", identifier.0)?,
            MirTerminator::SwitchInteger {
                discriminant,
                targets,
                otherwise,
            } => {
                let element = self.operand(body, discriminant, None)?;
                let writer = width(&element.value_type)
                    .ok_or_else(backend_messages::c_emission_noninteger_switch)?;

                for (count, identifier) in targets {
                    writeln!(
                        output,
                        "if(({})=={})goto b{};",
                        element.code,
                        bits(&format!("{}ULL", *count as u64), writer),
                        identifier.0
                    )?;
                }

                writeln!(output, "goto b{};", otherwise.0)?
            }
            MirTerminator::Call {
                function: current_function,
                arguments,
                destination,
                target,
            } => {
                let element = self.call(body, current_function, arguments)?;
                let value_type = body.place_type(destination);

                if element.value_type == Type::Void {
                    writeln!(output, "{};", element.code)?
                } else {
                    let sign_extend = signed(&element.value_type);
                    let current_value = convert(element, &value_type, sign_extend)?;

                    writeln!(
                        output,
                        "{}={};",
                        self.place(body, destination)?,
                        current_value.code
                    )?
                }

                writeln!(output, "goto b{};", target.0)?
            }
            MirTerminator::Throw { error } => {
                if !body.is_throws {
                    return Err(EmitError::internal(
                        backend_messages::c_emission_throw_outside_throws_body(),
                    ));
                }

                let value = self.operand(body, error, Some(&Type::String))?;
                writeln!(
                    output,
                    "return ({}){{0,{},{}}};",
                    throws_ctype(&return_value),
                    zero(&return_value)?,
                    value.code
                )?;
            }
            MirTerminator::CatchCall {
                function: current_function,
                arguments,
                fallback,
                destination,
                target,
            } => {
                let (call, payload_type) = self.throwing_call(body, current_function, arguments)?;
                let value_type = body.place_type(destination);

                if value_type == Type::Void {
                    return Err(backend_messages::c_emission_void_catch_destination().into());
                }

                let function = self.operand(body, fallback, Some(&value_type))?;
                let value = convert(
                    Value {
                        code: "_x_result.value".into(),
                        value_type: payload_type.clone(),
                    },
                    &value_type,
                    signed(&payload_type),
                )?;
                writeln!(
                    output,
                    "{{{} _x_result={call};{} _x_fallback={};{}=_x_result.ok?{}:_x_fallback;goto b{};}}",
                    throws_ctype(&payload_type),
                    ctype(&value_type)?,
                    function.code,
                    self.place(body, destination)?,
                    value.code,
                    target.0
                )?;
            }
            MirTerminator::TryCall {
                function: current_function,
                arguments,
                destination,
                target,
            } => {
                if !body.is_throws {
                    return Err(EmitError::internal(
                        backend_messages::c_emission_try_outside_throws_body(),
                    ));
                }

                let (call, payload_type) = self.throwing_call(body, current_function, arguments)?;
                writeln!(
                    output,
                    "{{{} _x_result={call};if(!_x_result.ok)return ({}){{0,{},_x_result.error}};",
                    throws_ctype(&payload_type),
                    throws_ctype(&return_value),
                    zero(&return_value)?
                )?;
                let value_type = body.place_type(destination);

                if value_type != Type::Void {
                    let value = convert(
                        Value {
                            code: "_x_result.value".into(),
                            value_type: payload_type.clone(),
                        },
                        &value_type,
                        signed(&payload_type),
                    )?;
                    writeln!(output, "{}={};", self.place(body, destination)?, value.code)?;
                }

                writeln!(output, "goto b{};}}", target.0)?;
            }
            MirTerminator::TailCall {
                function: current_function,
                arguments,
            } => {
                let Some(ResolvedCall::Source { name: callee, .. }) =
                    body.call_targets.get(current_function)
                else {
                    return Err(backend_messages::c_emission_non_source_tailcall().into());
                };

                if callee != name || body.is_throws || body.is_memoised {
                    return Err(backend_messages::c_emission_non_self_or_contract_tailcall().into());
                }

                let signature = self.symbols.get(name).ok_or_else(|| {
                    EmitError::internal(
                        backend_messages::c_emission_registered_function_is_missing(),
                    )
                })?;

                if arguments.len() != signature.parameters.len() {
                    return Err(EmitError::internal(
                        backend_messages::c_emission_tailcall_arity(),
                    ));
                }

                output.push_str("{\n");
                for (index, (o, type_value)) in
                    arguments.iter().zip(signature.parameters).enumerate()
                {
                    let value = self.operand(body, o, None)?;
                    let sign_extend = signed(&value.value_type);
                    let value = convert(value, type_value, sign_extend)?;
                    writeln!(
                        output,
                        "{} _x_tail{index}={};",
                        ctype(type_value)?,
                        value.code
                    )?;
                }

                for index in 0..arguments.len() {
                    writeln!(output, "v{}=_x_tail{index};", index + 1)?;
                }

                output.push_str("goto b0;\n}\n");
            }
            MirTerminator::Abort => output.push_str("_x_trap();\n"),
            MirTerminator::Exit { code } => {
                let value = self.operand(body, code, Some(&Type::I32))?;
                writeln!(output, "fflush((void*)0);exit({});", value.code)?;
            }
            MirTerminator::Unreachable => {
                return Err(backend_messages::c_emission_unreachable_terminator().into());
            }
        }

        Ok(())
    }
}
