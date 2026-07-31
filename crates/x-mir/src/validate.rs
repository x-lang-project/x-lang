mod block;
mod calls;
mod errors;
mod types;

use crate::{BlockIdentifier, FunctionMeta, GlobalItem, LocalIdentifier, MirBody, MirProgram};
use block::BlockValidator;
use calls::{validate_call_identities, validate_call_sites};
use types::validate_type;
use x_diagnostics::backend_messages;

pub use errors::MirValidationError;
impl MirProgram {
    pub fn validate(&self) -> Result<(), MirValidationError> {
        validate_items(&self.items)?;

        self.contract_obligations()
            .map(|_| ())
            .map_err(MirValidationError::ContractObligation)
    }
}

fn validate_items(items: &[GlobalItem]) -> Result<(), MirValidationError> {
    for item in items {
        match item {
            GlobalItem::Function {
                name,
                parameter_types,
                return_type,
                body,
                meta,
            } => {
                for (index, value_type) in parameter_types.iter().enumerate() {
                    validate_type(value_type, &|| {
                        backend_messages::function_name_parameter_index(name, index)
                    })?;
                }

                validate_type(return_type, &|| {
                    backend_messages::function_name_return_type(name)
                })?;

                if let Some(body) = body {
                    validate_function_contract(name, body, meta)?;
                    validate_body(name, body)?;
                }
            }
            GlobalItem::Static {
                name, value_type, ..
            } => {
                validate_type(value_type, &|| backend_messages::static_name(name))?;
            }
            GlobalItem::Struct(definition) => {
                for (field, value_type) in &definition.fields {
                    validate_type(value_type, &|| {
                        backend_messages::struct_value_field_field(&definition.name, field)
                    })?;
                }
            }
            GlobalItem::Impl { methods, .. } => validate_items(methods)?,
            GlobalItem::Trait(_) => {}
        }
    }

    Ok(())
}

fn validate_function_contract(
    function: &str,
    body: &MirBody,
    meta: &FunctionMeta,
) -> Result<(), MirValidationError> {
    if body.is_throws != meta.is_throws
        || body.is_memoised != meta.is_memoised
        || body.call_contract != meta.call_contract
    {
        return Err(MirValidationError::FunctionContractMismatch {
            function: function.to_owned(),
        });
    }

    Ok(())
}

fn validate_body(function: &str, body: &MirBody) -> Result<(), MirValidationError> {
    // Type lookup requires validated locals and places.
    validate_call_identities(function, body)?;
    validate_frame(function, body)?;

    for (block_identifier, block) in &body.basic_blocks {
        BlockValidator {
            function,
            block: *block_identifier,
            body,
        }
        .validate_block(block)?;
    }

    validate_call_sites(function, body)
}

fn validate_frame(function: &str, body: &MirBody) -> Result<(), MirValidationError> {
    if !body.basic_blocks.contains_key(&BlockIdentifier(0)) {
        return Err(MirValidationError::MissingEntryBlock {
            function: function.to_string(),
        });
    }

    if !body.locals.contains_key(&body.return_local) {
        return Err(MirValidationError::MissingReturnLocal {
            function: function.to_string(),
            local: body.return_local.0,
        });
    }

    for argument in 1..=body.argument_count {
        if !body.locals.contains_key(&LocalIdentifier(argument)) {
            return Err(MirValidationError::MissingArgumentLocal {
                function: function.to_string(),
                argument,
            });
        }
    }

    for (identifier, local) in &body.locals {
        if *identifier != local.identifier {
            return Err(MirValidationError::LocalIdentityMismatch {
                function: function.to_string(),
                key: identifier.0,
                declared: local.identifier.0,
            });
        }

        validate_type(&local.value_type, &|| {
            backend_messages::function_function_local_value(function, identifier.0)
        })?;
    }

    Ok(())
}
