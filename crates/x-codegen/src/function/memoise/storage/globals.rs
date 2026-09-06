use super::*;

pub(super) fn memo_global<'ctx>(
    compiler: &FunctionCompiler<'_, 'ctx>,
    name: &str,
    value_type: BasicTypeEnum<'ctx>,
) -> PointerValue<'ctx> {
    let global = compiler.m().get_global(name).unwrap_or_else(|| {
        let global = compiler.m().add_global(
            value_type,
            Some(compiler.codegen.pointer_address_space()),
            name,
        );
        global.set_linkage(Linkage::Internal);
        global.set_initializer(&value_type.const_zero());

        global
    });

    global.as_pointer_value()
}

pub(super) fn memo_array_global<'ctx>(
    compiler: &FunctionCompiler<'_, 'ctx>,
    name: &str,
    element: BasicTypeEnum<'ctx>,
    capacity: u32,
) -> PointerValue<'ctx> {
    memo_global(compiler, name, element.array_type(capacity).into())
}
