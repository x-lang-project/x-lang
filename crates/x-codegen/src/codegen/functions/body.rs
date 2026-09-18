use std::collections::HashMap;

use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::basic_block;
use inkwell::values::{FunctionValue, InstructionOpcode};

use crate::codegen::CodeGeneration;
use crate::errors::CodeGenerationError;
use crate::function::FunctionCompiler;
use x_diagnostics::backend_messages;
use x_mir::{BlockIdentifier, MirBody};

impl<'ctx> CodeGeneration<'ctx> {
    pub(in crate::codegen) fn prepare_function_compiler<'a>(
        &'a self,
        function_name: &'a str,
        body: &'a MirBody,
        block_order: &[usize],
    ) -> Result<(FunctionValue<'ctx>, FunctionCompiler<'a, 'ctx>), CodeGenerationError> {
        let llvm_name = self.llvm_function_symbol(function_name);
        let function = self
            .module
            .get_function(llvm_name)
            .ok_or_else(|| CodeGenerationError::FunctionSignatureMissing(llvm_name.to_string()))?;
        self.enter_alloca_block(function);
        let llvm_blocks = self.declare_llvm_blocks(function, block_order);
        let frame = self.allocate_frame_slots(body)?;
        self.bind_function_parameters(function, body, &frame)?;

        let compiler = FunctionCompiler {
            codegen: self,
            function_name,
            body,
            frame,
            llvm_blocks,
        };

        Ok((function, compiler))
    }

    pub(in crate::codegen) fn enter_alloca_block(&self, function: FunctionValue<'ctx>) {
        let entry_alloca_basic_block = self.context.append_basic_block(function, "entry_allocas");

        self.builder.position_at_end(entry_alloca_basic_block);
    }

    pub(in crate::codegen) fn sorted_basic_block_identifiers(body: &MirBody) -> Vec<usize> {
        let mut identifiers: Vec<usize> = body.basic_blocks.keys().map(|block| block.0).collect();
        identifiers.sort_unstable();

        identifiers
    }

    pub(in crate::codegen) fn declare_llvm_blocks(
        &self,
        function: FunctionValue<'ctx>,
        block_identifiers: &[usize],
    ) -> HashMap<usize, basic_block::BasicBlock<'ctx>> {
        let mut blocks = HashMap::with_capacity(block_identifiers.len());

        for &block_identifier in block_identifiers {
            let block = self
                .context
                .append_basic_block(function, &format!("bb_{block_identifier}"));
            blocks.insert(block_identifier, block);
        }

        blocks
    }

    pub(in crate::codegen) fn finalize_function_body(
        &self,
        function: FunctionValue<'ctx>,
        function_name: &str,
    ) -> Result<(), CodeGenerationError> {
        let obligations = self.contract_obligations.borrow();
        let certified = obligations.functions.get(function_name);
        let fallback_has_calls = function.get_basic_blocks().iter().any(|block| {
            block.get_instructions().any(|instruction| {
                matches!(
                    instruction.get_opcode(),
                    InstructionOpcode::Call | InstructionOpcode::CallBr | InstructionOpcode::Invoke
                )
            })
        });
        let permits_no_memory = certified.is_some_and(|entry| {
            entry.source_pure && entry.memory.permits_memory_none() && !fallback_has_calls
        });

        if certified.is_some() || fallback_has_calls {
            function.remove_enum_attribute(
                AttributeLoc::Function,
                Attribute::get_named_enum_kind_id("memory"),
            );
        }

        if permits_no_memory {
            function.add_attribute(
                AttributeLoc::Function,
                self.context
                    .create_enum_attribute(Attribute::get_named_enum_kind_id("memory"), 0),
            );
        }

        if cfg!(debug_assertions) && !function.verify(true) {
            return Err(CodeGenerationError::LlvmVerificationFailure(
                backend_messages::generated_function_function_name_failed_llvm_verification(
                    function_name,
                ),
            ));
        }

        Ok(())
    }
}

impl FunctionCompiler<'_, '_> {
    pub(in crate::codegen) fn compile_body(
        &mut self,
        block_order: &[usize],
    ) -> Result<(), CodeGenerationError> {
        self.emit_function_entry()?;

        for &block_identifier in block_order {
            let block = self
                .body
                .basic_blocks
                .get(&BlockIdentifier(block_identifier))
                .ok_or(CodeGenerationError::MissingMirBlock(block_identifier))?;
            let llvm_block = *self
                .llvm_blocks
                .get(&block_identifier)
                .ok_or(CodeGenerationError::MissingLlvmBlock(block_identifier))?;
            self.codegen.builder.position_at_end(llvm_block);
            self.compile_block_statements(&block.statements)?;
            let terminator = block
                .terminator
                .as_ref()
                .ok_or(CodeGenerationError::MissingTerminator(block_identifier))?;
            self.compile_terminator(terminator)?;
        }

        Ok(())
    }

    fn emit_function_entry(&mut self) -> Result<(), CodeGenerationError> {
        let entry = *self
            .llvm_blocks
            .get(&0)
            .ok_or(CodeGenerationError::MissingEntryBlock)?;

        if self.body.is_memoised {
            self.emit_memo_lookup(entry)?;
        } else {
            self.codegen.builder.build_unconditional_branch(entry)?;
        }

        Ok(())
    }
}
