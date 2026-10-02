use inkwell::context::Context;

pub(crate) fn object_context(discard_value_names: bool) -> Context {
    let context = Context::create();

    if discard_value_names {
        use inkwell::context::AsContextRef;

        // SAFETY: the context is live, uniquely owned here, and contains no
        // modules yet. LLVM retains global names required for symbol linkage.
        unsafe {
            inkwell::llvm_sys::core::LLVMContextSetDiscardValueNames((&context).as_ctx_ref(), 1);
        }
    }

    context
}
