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
fn memo_symbol(name: &str) -> String {
    let mut output = String::new();

    for byte in name.bytes() {
        if byte.is_ascii_alphanumeric() {
            output.push(char::from(byte));
        } else {
            output.push_str(&format!("_{byte:02x}"));
        }
    }

    output
}

struct PreparedMemo<'ctx> {
    return_type: BasicTypeEnum<'ctx>,
    key_bits: Vec<IntValue<'ctx>>,
    index: IntValue<'ctx>,
    state: MemoState<'ctx>,
}

struct MemoState<'ctx> {
    lock: PointerValue<'ctx>,
    valid: PointerValue<'ctx>,
    keys: Vec<PointerValue<'ctx>>,
    values: PointerValue<'ctx>,
}
