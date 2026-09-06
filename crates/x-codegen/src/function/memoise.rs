mod storage;
use inkwell::basic_block;
use inkwell::{
    AtomicOrdering, IntPredicate,
    module::Linkage,
    types::{BasicType, BasicTypeEnum},
    values::{BasicValueEnum, IntValue, PointerValue},
};

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::LocalIdentifier;
use x_mir::{
    CallContract, MemoiseCallContract, MemoiseIndexing, MemoiseKeyEncoding, MemoiseSynchronization,
};

// Escape '_' too, so source names cannot collide.
