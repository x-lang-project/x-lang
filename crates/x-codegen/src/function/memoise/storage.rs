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
