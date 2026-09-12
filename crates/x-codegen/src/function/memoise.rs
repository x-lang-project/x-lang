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

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    fn prepare_memo(&self) -> Result<PreparedMemo<'ctx>, CodeGenerationError> {
        self.validate_memo_contract()?;
        let return_type = self.memo_return_type()?;
        let key_bits = self.memo_key_bits()?;
        let index = self.memo_index(&key_bits)?;
        let return_type = self.codegen.resolve_llvm_type(&return_type)?;
        let state = self.memo_state(return_type, key_bits.len());

        Ok(PreparedMemo {
            return_type,
            key_bits,
            index,
            state,
        })
    }

    pub(crate) fn emit_memo_lookup(
        &self,
        miss_target: basic_block::BasicBlock<'ctx>,
    ) -> Result<(), CodeGenerationError> {
        let PreparedMemo {
            return_type,
            key_bits,
            index,
            state,
        } = self.prepare_memo()?;

        self.acquire_memo_lock(state.lock)?;
        let valid_pointer = self.memo_array_slot(
            state.valid,
            self.boolean_type().into(),
            index,
            "memo_valid_ptr",
        )?;
        let mut hit = self
            .load(self.boolean_type().into(), valid_pointer, "memo_valid")?
            .into_int_value();

        for (key, bits) in state.keys.iter().zip(&key_bits) {
            let key_pointer =
                self.memo_array_slot(*key, self.i64().into(), index, "memo_key_ptr")?;
            let cached = self
                .load(self.i64().into(), key_pointer, "memo_cached_key")?
                .into_int_value();
            let equal = self.icmp(IntPredicate::EQ, cached, *bits, "memo_key_equal")?;
            hit = self.b().build_and(hit, equal, "memo_hit")?;
        }

        let function = self.current_function()?;
        let hit_block = self.context().append_basic_block(function, "memo_hit");
        let miss_block = self.context().append_basic_block(function, "memo_miss");

        self.condition_br(hit, hit_block, miss_block)?;

        self.b().position_at_end(hit_block);
        let value_pointer =
            self.memo_array_slot(state.values, return_type, index, "memo_value_ptr")?;
        let value = self.load(return_type, value_pointer, "memo_value")?;

        self.release_memo_lock(state.lock)?;
        self.return_value(Some(&value))?;

        self.b().position_at_end(miss_block);
        self.release_memo_lock(state.lock)?;

        self.br(miss_target)
    }

    pub(super) fn emit_memo_store(&self) -> Result<(), CodeGenerationError> {
        let PreparedMemo {
            return_type,
            key_bits,
            index,
            state,
        } = self.prepare_memo()?;
        let return_pointer = self.frame.pointer(self.body.return_local).ok_or_else(|| {
            CodeGenerationError::LlvmInternalError(
                backend_messages::memoised_return_slot_is_missing(),
            )
        })?;
        let value = self.load(return_type, return_pointer, "memo_store_value")?;

        self.acquire_memo_lock(state.lock)?;
        for (key, bits) in state.keys.iter().zip(&key_bits) {
            let key_pointer =
                self.memo_array_slot(*key, self.i64().into(), index, "memo_store_key_ptr")?;
            self.store(key_pointer, (*bits).into())?;
        }

        let value_pointer =
            self.memo_array_slot(state.values, return_type, index, "memo_store_value_ptr")?;
        self.store(value_pointer, value)?;
        let valid_pointer = self.memo_array_slot(
            state.valid,
            self.boolean_type().into(),
            index,
            "memo_store_valid_ptr",
        )?;
        self.store(
            valid_pointer,
            self.boolean_type().const_int(1, false).into(),
        )?;

        self.release_memo_lock(state.lock)
    }
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
