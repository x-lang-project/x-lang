use super::*;

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
