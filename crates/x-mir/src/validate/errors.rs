use std::fmt::{Display, Formatter};
use std::{error, fmt};

use x_ast::Type;
use x_diagnostics::backend_messages;

#[derive(Debug, PartialEq, Eq)]
pub enum MirValidationError {
    ContractObligation(crate::ContractObligationError),
    FunctionContractMismatch {
        function: String,
    },
    MissingEntryBlock {
        function: String,
    },
    MissingReturnLocal {
        function: String,
        local: usize,
    },
    MissingArgumentLocal {
        function: String,
        argument: usize,
    },
    LocalIdentityMismatch {
        function: String,
        key: usize,
        declared: usize,
    },
    MissingTerminator {
        function: String,
        block: usize,
    },
    MissingTargetBlock {
        function: String,
        block: usize,
        target: usize,
    },
    MissingLocal {
        function: String,
        block: usize,
        local: usize,
    },
    UnresolvedType {
        context: String,
        value_type: Type,
    },
    ByteCharacterOutOfRange {
        function: String,
        block: usize,
    },
    NonFiniteFloat {
        function: String,
        block: usize,
    },
    CallIdentityMismatch {
        function: String,
        call: String,
    },
}

impl Display for MirValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContractObligation(error) => error.fmt(formatter),
            Self::FunctionContractMismatch { function } => formatter
                .write_str(&backend_messages::mir_validation_function_contract_mismatch(function)),
            Self::MissingEntryBlock { function } => formatter.write_str(
                &backend_messages::mir_validation_missing_entry_block(function),
            ),
            Self::MissingReturnLocal { function, local } => formatter.write_str(
                &backend_messages::mir_validation_missing_return_local(function, local),
            ),
            Self::MissingArgumentLocal { function, argument } => formatter.write_str(
                &backend_messages::mir_validation_missing_argument_local(function, argument),
            ),
            Self::LocalIdentityMismatch {
                function,
                key,
                declared,
            } => formatter.write_str(&backend_messages::mir_validation_local_identity_mismatch(
                function, key, declared,
            )),
            Self::MissingTerminator { function, block } => formatter.write_str(
                &backend_messages::mir_validation_missing_terminator(function, block),
            ),
            Self::MissingTargetBlock {
                function,
                block,
                target,
            } => formatter.write_str(&backend_messages::mir_validation_missing_target_block(
                function, block, target,
            )),
            Self::MissingLocal {
                function,
                block,
                local,
            } => formatter.write_str(&backend_messages::mir_validation_missing_local(
                function, block, local,
            )),
            Self::UnresolvedType {
                context,
                value_type,
            } => formatter.write_str(&backend_messages::mir_validation_unresolved_type(
                context, value_type,
            )),
            Self::ByteCharacterOutOfRange { function, block } => formatter.write_str(
                &backend_messages::mir_validation_byte_character_output_of_range(function, block),
            ),
            Self::NonFiniteFloat { function, block } => formatter.write_str(
                &backend_messages::mir_validation_non_finite_float(function, block),
            ),
            Self::CallIdentityMismatch { function, call } => formatter.write_str(
                &backend_messages::mir_validation_call_identity_mismatch(function, call),
            ),
        }
    }
}

impl error::Error for MirValidationError {}
