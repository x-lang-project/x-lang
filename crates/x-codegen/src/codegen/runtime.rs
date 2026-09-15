use inkwell::module::Linkage;
use inkwell::types::FunctionType;
use inkwell::values::FunctionValue;

use super::CodeGeneration;
use x_runtime::abi;

impl<'ctx> CodeGeneration<'ctx> {
    pub(super) fn ensure_optional_main(&self) {
        if self.module.get_function(x_ast::ENTRY_POINT_NAME).is_some() {
            return;
        }

        let function_type = self.context.i32_type().fn_type(&[], false);
        let function = self.module.add_function(
            x_ast::ENTRY_POINT_NAME,
            function_type,
            Some(Linkage::WeakAny),
        );
        let entry = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(entry);
        let _ = self
            .builder
            .build_return(Some(&self.context.i32_type().const_zero()));
    }

    pub(super) fn register_stdlib_stubs(&self) {
        let pointer = self.pointer_type();
        let usize = self.context.ptr_sized_int_type(&self.target_data(), None);
        let i32 = self.context.i32_type();
        let i8 = self.context.i8_type();
        let void = self.context.void_type();

        for (name, signature) in [
            ("putchar", i32.fn_type(&[i32.into()], false)),
            ("printf", i32.fn_type(&[pointer.into()], true)),
            (abi::VERSION, i32.fn_type(&[], false)),
            (abi::ALLOC, pointer.fn_type(&[usize.into()], false)),
            (
                abi::REALLOC,
                pointer.fn_type(&[pointer.into(), usize.into()], false),
            ),
            (abi::FREE, void.fn_type(&[pointer.into()], false)),
            (abi::STRING_LEN, usize.fn_type(&[pointer.into()], false)),
            (
                abi::WRITE_STDOUT,
                void.fn_type(&[pointer.into(), usize.into()], false),
            ),
            (abi::EXIT_CODE, i32.fn_type(&[i32.into()], false)),
            (abi::PROCESS_EXIT, void.fn_type(&[i32.into()], false)),
            (abi::PANIC, void.fn_type(&[pointer.into()], false)),
        ] {
            self.ensure_external_function(name, signature);
        }

        self.parallel_dispatch_function(false);
        self.parallel_dispatch_function(true);

        for name in ["intrinsic::memcpy", "intrinsic::memmove"] {
            self.ensure_external_function(
                name,
                void.fn_type(&[pointer.into(), pointer.into(), usize.into()], false),
            );
        }

        self.ensure_external_function(
            "intrinsic::memset",
            void.fn_type(&[pointer.into(), i8.into(), usize.into()], false),
        );
    }

    pub(crate) fn parallel_dispatch_function(&self, with_context: bool) -> FunctionValue<'ctx> {
        let name = if with_context {
            abi::PARALLEL_FOR_CONTEXT
        } else {
            abi::PARALLEL_FOR
        };

        if let Some(function) = self.module.get_function(name) {
            return function;
        }

        let index = self.context.i32_type();
        let pointer = self.pointer_type();
        let void = self.context.void_type();
        let signature = if with_context {
            void.fn_type(
                &[index.into(), index.into(), pointer.into(), pointer.into()],
                false,
            )
        } else {
            void.fn_type(&[index.into(), index.into(), pointer.into()], false)
        };

        self.ensure_external_function(name, signature)
    }

    pub(crate) fn ensure_external_function(
        &self,
        name: &str,
        function_type: FunctionType<'ctx>,
    ) -> FunctionValue<'ctx> {
        self.module.get_function(name).unwrap_or_else(|| {
            self.module
                .add_function(name, function_type, Some(Linkage::External))
        })
    }
}
