use super::*;
use x_ast::intrinsics::BuiltinCall;
use x_ast::intrinsics::BuiltinKind;
use x_diagnostics::backend_messages;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn emit_layout_allocator_or_memory_call(
        &self,
        call: DispatchCall<'_>,
        arguments: &[Operand],
    ) -> DispatchAttempt<'ctx> {
        if matches!(call.identity.kind, BuiltinKind::CoreMemory(_)) {
            return self
                .emit_core_mem_method(call.identity, arguments)
                .map(CallEmission::emitted);
        }

        if matches!(call.identity.kind, BuiltinKind::HeapAllocate(_)) {
            return self
                .emit_heap_allocate_call(call.identity, arguments)
                .map(CallEmission::Value);
        }

        if matches!(
            call.identity.kind,
            BuiltinKind::Intrinsic(intrinsics::Intrinsic::Layout(_))
        ) {
            return self
                .emit_compile_time_value(call.identity, arguments)
                .map(|value| value.map_or(CallEmission::Unsupported, CallEmission::Value));
        }

        if call.identity.kind == BuiltinKind::MemSwap {
            self.emit_mem_swap(call.identity, arguments)?;

            return Ok(CallEmission::Void);
        }

        if call.identity.kind == BuiltinKind::MemReplace {
            return self
                .emit_mem_replace(call.identity, arguments)
                .map(CallEmission::Value);
        }

        if call.identity.kind == BuiltinKind::AssertNonNull {
            let argument = arguments.first().ok_or_else(|| {
                CodeGenerationError::LlvmInternalError(
                    backend_messages::pointer_assert_non_null_expects_a_pointer(),
                )
            })?;
            let pointer = self.load_operand_value(argument)?;

            if !pointer.is_pointer_value() {
                return Err(CodeGenerationError::LlvmInternalError(
                    backend_messages::pointer_assert_non_null_received_a_non_pointer_value(),
                ));
            }

            let non_null = self
                .b()
                .build_is_not_null(pointer.into_pointer_value(), "assert_non_null_check")?;
            self.trap_if_false(non_null, "assert_non_null")?;

            return Ok(CallEmission::Void);
        }

        if call.identity.kind == BuiltinKind::AssertAligned {
            return Err(CodeGenerationError::LlvmInternalError(backend_messages::pointer_assert_aligned_has_no_declared_source_level_contract_or_backend_implementation()));
        }

        Ok(CallEmission::Unsupported)
    }

    pub(super) fn emit_heap_allocate_call(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let function = call.name.as_str();
        let size_argument = arguments.first().ok_or_else(|| {
            CodeGenerationError::LlvmInternalError(
                backend_messages::function_expects_a_size_argument(function),
            )
        })?;
        let size = self.load_operand_value_with_type(size_argument, &Type::USize)?;
        let allocate_name = x_runtime::abi::ALLOC;
        let malloc = self.required_function(allocate_name)?;
        let emitted_call = self.call(malloc, &[size.into()], "heap_alloc")?;
        let pointer = emitted_call.try_as_basic_value().basic().ok_or_else(|| {
            CodeGenerationError::LlvmInternalError(backend_messages::allocate_name_returned_void(
                allocate_name,
            ))
        })?;

        if call.kind == BuiltinKind::HeapAllocate(intrinsics::HeapAllocation::Zeroed) {
            let memset = self.required_function("intrinsic::memset")?;
            self.call(
                memset,
                &[
                    pointer.into(),
                    self.context().i8_type().const_zero().into(),
                    size.into(),
                ],
                "heap_alloc_zeroed",
            )?;
        }

        Ok(pointer)
    }

    pub(super) fn emit_compile_time_value(
        &self,
        call: &BuiltinCall,
        arguments: &[Operand],
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenerationError> {
        let function = call.name.as_str();
        let BuiltinKind::Intrinsic(intrinsics::Intrinsic::Layout(operation)) = call.kind else {
            return Ok(None);
        };

        if matches!(
            operation,
            x_ast::LayoutIntrinsic::SizeOf | x_ast::LayoutIntrinsic::AlignOf
        ) {
            let value_type = call.type_argument.as_ref().ok_or_else(|| {
                CodeGenerationError::InvalidMir(
                    backend_messages::layout_query_function_requires_a_concrete_type(function),
                )
            })?;
            let (size, align) = self.codegen.resolve_type_layout(value_type)?;
            let value = if operation == x_ast::LayoutIntrinsic::SizeOf {
                size
            } else {
                align
            };

            return Ok(Some(
                self.codegen
                    .resolve_llvm_type(&Type::USize)?
                    .into_int_type()
                    .const_int(value, false)
                    .into(),
            ));
        }

        let [Operand::ConstantString(field_name)] = arguments else {
            return Err(CodeGenerationError::InvalidMir(
                backend_messages::function_requires_exactly_one_compile_time_field_token(function),
            ));
        };

        let value_type = call.type_argument.as_ref().ok_or_else(|| {
            CodeGenerationError::InvalidMir(
                backend_messages::layout_query_function_requires_a_concrete_type(function),
            )
        })?;
        let offset = self
            .codegen
            .resolve_named_field_offset(&value_type.to_string(), field_name)?;

        Ok(Some(
            self.codegen
                .resolve_llvm_type(&Type::USize)?
                .into_int_type()
                .const_int(offset, false)
                .into(),
        ))
    }
}
