use super::{EmitError, Emitter, Result, Signature, ctype};
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::{GlobalItem, LocalIdentifier};

impl<'a> Emitter<'a> {
    pub(super) fn register(&mut self, items: &'a [GlobalItem]) -> Result<()> {
        for item in items {
            match item {
                GlobalItem::Function {
                    name,
                    parameter_types,
                    return_type,
                    body,
                    meta,
                } => {
                    if name == "main"
                        && (return_type != &Type::I32
                            || !parameter_types.is_empty()
                            || meta.is_throws
                            || meta.is_memoised)
                    {
                        return Err(backend_messages::c_emission_unsupported_main_abi().into());
                    }

                    if meta.is_exported && name != "main" {
                        return Err(backend_messages::c_emission_exported_abi_function(name).into());
                    }

                    if !meta.is_memoised && meta.call_contract.is_some() {
                        return Err(
                            backend_messages::c_emission_unknown_nonmemoise_call_contract().into(),
                        );
                    }

                    let Some(body) = body else {
                        return Err(backend_messages::c_emission_external_function(name).into());
                    };

                    if body.argument_count != parameter_types.len()
                        || body
                            .locals
                            .get(&body.return_local)
                            .is_none_or(|local| &local.value_type != return_type)
                        || parameter_types
                            .iter()
                            .enumerate()
                            .any(|(index, value_type)| {
                                body.locals
                                    .get(&LocalIdentifier(index + 1))
                                    .is_none_or(|local| &local.value_type != value_type)
                            })
                    {
                        return Err(EmitError::internal(
                            backend_messages::c_emission_function_signature_body_mismatch_for(name),
                        ));
                    }

                    if self.symbols.contains_key(name.as_str()) {
                        return Err(EmitError::internal(
                            backend_messages::c_emission_duplicate_function_symbol(name),
                        ));
                    }

                    ctype(return_type)?;
                    for type_value in parameter_types {
                        ctype(type_value)?;
                    }

                    let symbol = if name == "main" {
                        "main".to_owned()
                    } else if meta.is_exported {
                        if !name
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                            || name.as_bytes().first().is_none_or(u8::is_ascii_digit)
                        {
                            return Err(backend_messages::c_emission_export_name(name).into());
                        }

                        name.clone()
                    } else {
                        format!("_x_fn{}", self.order.len())
                    };

                    self.order.push(name);
                    self.symbols.insert(
                        name,
                        Signature {
                            symbol,
                            parameters: parameter_types,
                            return_value: return_type,
                            body,
                        },
                    );
                }
                GlobalItem::Impl { methods, .. } => self.register(methods)?,
                GlobalItem::Trait(_) => {}
                GlobalItem::Struct(definition) => {
                    if definition.invariant.is_some() {
                        return Err(backend_messages::c_emission_struct_runtime_invariant().into());
                    }

                    if definition.layout != x_ast::Layout::AoS {
                        return Err(backend_messages::c_emission_nonaos_layout().into());
                    }

                    if definition
                        .generic_parameters
                        .as_ref()
                        .is_some_and(|pointer| !pointer.is_empty())
                    {
                        return Err(backend_messages::c_emission_generic_record_definition().into());
                    }

                    self.structs.insert(&definition.name, definition);
                }
                GlobalItem::Static { .. } => {
                    return Err(backend_messages::c_emission_static_declaration().into());
                }
            }
        }

        Ok(())
    }
}
