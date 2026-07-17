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
