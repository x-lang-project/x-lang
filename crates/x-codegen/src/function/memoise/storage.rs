mod globals;
use super::*;
use globals::*;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn memo_return_type(&self) -> Result<Type, CodeGenerationError> {
        let value_type = self
            .body
            .locals
            .get(&self.body.return_local)
            .map(|declaration| declaration.value_type.clone())
            .ok_or_else(|| {
                CodeGenerationError::LlvmInternalError(
                    backend_messages::memoised_return_type_is_missing(),
                )
            })?;

        if !Type::is_scalar(&value_type) {
            return Err(CodeGenerationError::UnsupportedType(value_type));
        }

        Ok(value_type)
    }

    pub(super) fn memo_key_bits(&self) -> Result<Vec<IntValue<'ctx>>, CodeGenerationError> {
        let mut bits = Vec::with_capacity(self.body.argument_count);

        for index in 0..self.body.argument_count {
            let local = self
                .body
                .locals
                .get(&LocalIdentifier(index + 1))
                .ok_or_else(|| {
                    CodeGenerationError::LlvmInternalError(
                        backend_messages::memoised_parameter_layout_is_missing(),
                    )
                })?;

            if !Type::is_scalar(&local.value_type) {
                return Err(CodeGenerationError::UnsupportedType(
                    local.value_type.clone(),
                ));
            }

            let current_pointer =
                self.frame
                    .pointer(LocalIdentifier(index + 1))
                    .ok_or_else(|| {
                        CodeGenerationError::LlvmInternalError(
                            backend_messages::memoised_parameter_slot_is_missing(),
                        )
                    })?;
            let llvm_type = self.codegen.resolve_llvm_type(&local.value_type)?;
            let value = self.load(llvm_type, current_pointer, "memo_key_value")?;
            bits.push(self.memo_value_bits(value)?);
        }

        Ok(bits)
    }

    pub(super) fn memo_value_bits(
        &self,
        value: BasicValueEnum<'ctx>,
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        let raw = if value.is_int_value() {
            value.into_int_value()
        } else if value.is_float_value() {
            let float = value.into_float_value();
            let integer_type = if float.get_type() == self.context().f32_type() {
                self.i32()
            } else {
                self.i64()
            };

            self.b()
                .build_bit_cast(float, integer_type, "memo_float_bits")?
                .into_int_value()
        } else {
            return Err(CodeGenerationError::LlvmInternalError(
                backend_messages::memoisation_key_is_not_a_scalar_bit_value(),
            ));
        };

        self.b()
            .build_int_z_extend_or_bit_cast(raw, self.i64(), "memo_key_bits")
            .map_err(Into::into)
    }

    pub(super) fn memo_index(
        &self,
        keys: &[IntValue<'ctx>],
    ) -> Result<IntValue<'ctx>, CodeGenerationError> {
        let mut hash = self.i64().const_int(0xcbf29ce484222325, false);
        let prime = self.i64().const_int(0x100000001b3, false);

        for key in keys {
            hash = self.b().build_xor(hash, *key, "memo_hash_xor")?;
            hash = self.b().build_int_mul(hash, prime, "memo_hash_mul")?;
        }
        // MurmurHash3 finaliser mixes key bits before cache indexing.
        for (shift, multiplier) in [
            (33, Some(0xff51afd7ed558ccd_u64)),
            (33, Some(0xc4ceb9fe1a85ec53_u64)),
            (33, None),
        ] {
            let upper = self.b().build_right_shift(
                hash,
                self.i64().const_int(shift, false),
                false,
                "memo_hash_upper",
            )?;
            hash = self.b().build_xor(hash, upper, "memo_hash_fold")?;

            if let Some(multiplier) = multiplier {
                hash = self.b().build_int_mul(
                    hash,
                    self.i64().const_int(multiplier, false),
                    "memo_hash_avalanche",
                )?;
            }
        }

        self.b()
            .build_int_unsigned_rem(
                hash,
                self.i64()
                    .const_int(u64::from(self.memo_contract().capacity), false),
                "memo_index",
            )
            .map_err(Into::into)
    }

    pub(super) fn acquire_memo_lock(
        &self,
        lock: PointerValue<'ctx>,
    ) -> Result<(), CodeGenerationError> {
        let function = self.current_function()?;
        let loop_block = self
            .context()
            .append_basic_block(function, "memo_lock_wait");
        let acquired = self
            .context()
            .append_basic_block(function, "memo_lock_acquired");

        self.br(loop_block)?;

        self.b().position_at_end(loop_block);
        let exchange = self.b().build_cmpxchg(
            lock,
            self.i8().const_zero(),
            self.i8().const_int(1, false),
            AtomicOrdering::Acquire,
            AtomicOrdering::Monotonic,
        )?;
        let succeeded = self
            .b()
            .build_extract_value(exchange, 1, "memo_lock_success")?
            .into_int_value();
        self.condition_br(succeeded, acquired, loop_block)?;

        self.b().position_at_end(acquired);

        Ok(())
    }

    pub(super) fn release_memo_lock(
        &self,
        lock: PointerValue<'ctx>,
    ) -> Result<(), CodeGenerationError> {
        self.b()
            .build_store(lock, self.i8().const_zero())?
            .set_atomic_ordering(AtomicOrdering::Release)
            .map_err(|error| CodeGenerationError::LlvmInternalError(error.to_string()))?;

        Ok(())
    }

    pub(super) fn memo_array_slot(
        &self,
        global: PointerValue<'ctx>,
        element_type: BasicTypeEnum<'ctx>,
        index: IntValue<'ctx>,
        name: &str,
    ) -> Result<PointerValue<'ctx>, CodeGenerationError> {
        let current_array_type = element_type.array_type(self.memo_contract().capacity);

        unsafe {
            self.b()
                .build_gep(
                    current_array_type,
                    global,
                    &[self.i64().const_zero(), index],
                    name,
                )
                .map_err(Into::into)
        }
    }

    pub(super) fn memo_state(
        &self,
        return_type: BasicTypeEnum<'ctx>,
        key_count: usize,
    ) -> MemoState<'ctx> {
        let prefix = format!("__x_memo_{}", memo_symbol(self.function_name));
        let lock = memo_global(self, &format!("{prefix}_lock"), self.i8().into());
        let capacity = self.memo_contract().capacity;
        let valid = memo_array_global(
            self,
            &format!("{prefix}_valid"),
            self.boolean_type().into(),
            capacity,
        );

        let mut keys = Vec::with_capacity(key_count);

        for index in 0..key_count {
            keys.push(memo_array_global(
                self,
                &format!("{prefix}_key_{index}"),
                self.i64().into(),
                capacity,
            ));
        }

        let values = memo_array_global(self, &format!("{prefix}_values"), return_type, capacity);

        MemoState {
            lock,
            valid,
            keys,
            values,
        }
    }

    pub(super) fn memo_contract(&self) -> MemoiseCallContract {
        match &self.body.call_contract {
            Some(CallContract::Memoise(contract)) => contract.clone(),
            None => MemoiseCallContract::checked(x_mir::DEFAULT_MEMOISE_CAPACITY),
        }
    }

    pub(super) fn validate_memo_contract(&self) -> Result<(), CodeGenerationError> {
        let contract = self.memo_contract();

        if contract.capacity == 0
            || !contract.unlock_before_proceed
            || !contract.per_function_state
            || contract.key_encoding != MemoiseKeyEncoding::CompleteBitwiseU64
            || contract.indexing != MemoiseIndexing::UnsignedRemainder
            || contract.synchronization != MemoiseSynchronization::AcquireReleaseLock
        {
            return Err(CodeGenerationError::LlvmInternalError(
                backend_messages::invalid_typed_memoise_call_contract_reached_code_generation(),
            ));
        }

        Ok(())
    }
}
