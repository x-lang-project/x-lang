use super::*;

impl Emitter<'_> {
    pub(super) fn definition(&mut self, name: &str) -> Result<String> {
        let signature = self.symbols.get(name).ok_or_else(|| {
            EmitError::internal(backend_messages::c_emission_registered_function_is_missing())
        })?;
        let body = signature.body;

        if body.is_memoised {
            memoise::globals(&signature.symbol, signature)?;
        }

        let return_value = signature.return_value;
        let symbol = &signature.symbol;
        let mut output = String::new();
        let rt = if name == "main" {
            "int".to_owned()
        } else if body.is_throws {
            throws_ctype(return_value)
        } else {
            ctype(return_value)?
        };

        write!(output, "{rt} {symbol}(")?;
        for (index, type_value) in signature.parameters.iter().enumerate() {
            if index > 0 {
                output.push(',')
            }

            write!(output, "{} v{}", ctype(type_value)?, index + 1)?;
        }

        if signature.parameters.is_empty() {
            output.push_str("void")
        }

        output.push_str("){\n");
        let mut locals: Vec<_> = body.locals.iter().collect();
        locals.sort_by_key(|(identifier, _)| identifier.0);
        for (identifier, declaration) in locals {
            if identifier.0 >= 1 && identifier.0 <= body.argument_count {
                continue;
            }

            if declaration.value_type == Type::Void {
                continue;
            }

            writeln!(
                output,
                "{} v{};",
                ctype(&declaration.value_type)?,
                identifier.0
            )?;
        }

        if body.is_memoised {
            output.push_str(&memoise::lookup(signature)?);
        }

        output.push_str("goto b0;\n");
        let mut blocks: Vec<_> = body.basic_blocks.iter().collect();
        blocks.sort_by_key(|(identifier, _)| identifier.0);
        let reachable = reachable_blocks(body)?;

        for (identifier, block) in blocks {
            if !reachable.contains(&identifier.0) {
                continue;
            }

            writeln!(output, "b{}:;", identifier.0)?;
            self.emit_statements(body, block, &mut output)?;
            self.emit_terminator(name, block, &mut output)?;
        }

        output.push_str("}\n");

        Ok(output)
    }
}

fn reachable_blocks(body: &MirBody) -> Result<BTreeSet<usize>> {
    let mut reachable = BTreeSet::new();
    let mut pending = vec![BlockIdentifier(0)];

    while let Some(identifier) = pending.pop() {
        if !reachable.insert(identifier.0) {
            continue;
        }

        let block = body.basic_blocks.get(&identifier).ok_or_else(|| {
            EmitError::internal(backend_messages::c_emission_missing_reachable_block())
        })?;

        match block.terminator.as_ref().ok_or_else(|| {
            EmitError::internal(backend_messages::c_emission_missing_reachable_terminator())
        })? {
            MirTerminator::Goto(type_value)
            | MirTerminator::Call {
                target: type_value, ..
            }
            | MirTerminator::TryCall {
                target: type_value, ..
            }
            | MirTerminator::CatchCall {
                target: type_value, ..
            } => pending.push(*type_value),
            MirTerminator::SwitchInteger {
                targets, otherwise, ..
            } => {
                pending.push(*otherwise);
                pending.extend(targets.iter().map(|(_, identifier)| *identifier));
            }
            _ => {}
        }
    }

    Ok(reachable)
}

impl Emitter<'_> {
    fn emit_statements(
        &mut self,
        body: &MirBody,
        block: &BasicBlock,
        output: &mut String,
    ) -> Result<()> {
        for statement in &block.statements {
            match statement {
                MirStatement::Assign(pointer, result) => {
                    if string_element(body, pointer) {
                        return Err(backend_messages::c_emission_write_to_string_element().into());
                    }

                    let value_type = body.place_type(pointer);
                    let value = self.rvalue(body, result, &value_type)?;

                    if value.value_type == Type::Void {
                        writeln!(output, "{};", value.code)?;
                    } else {
                        let sign_extend = signed(&value.value_type);
                        let value = convert(value, &value_type, sign_extend)?;
                        writeln!(output, "{}={};", self.place(body, pointer)?, value.code)?;
                    }
                }
                MirStatement::StorageLive(_)
                | MirStatement::StorageDead(_)
                | MirStatement::Comment(_) => {}
                _ => return Err(backend_messages::c_emission_parallel_statement().into()),
            }
        }

        Ok(())
    }
}
