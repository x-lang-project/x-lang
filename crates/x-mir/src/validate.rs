mod block;
mod calls;
mod errors;
mod types;

use crate::BlockIdentifier;
use crate::FunctionMeta;
use crate::LocalIdentifier;
use crate::MirBody;
use block::BlockValidator;
use calls::validate_call_identities;
use types::validate_type;
use x_diagnostics::backend_messages;

pub use errors::MirValidationError;

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
