use super::*;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(in crate::function) fn gep(
        &self,
        element_type: BasicTypeEnum<'ctx>,
        base: PointerValue<'ctx>,
        index: IntValue<'ctx>,
        name: &str,
    ) -> Result<PointerValue<'ctx>, CodeGenerationError> {
        unsafe {
            self.b()
                .build_gep(element_type, base, &[index], name)
                .map_err(Into::into)
        }
    }

    pub(in crate::function) fn memcpy(
        &self,
        destination: PointerValue<'ctx>,
        source: PointerValue<'ctx>,
        length: IntValue<'ctx>,
    ) -> Result<(), CodeGenerationError> {
        self.b().build_memcpy(destination, 1, source, 1, length)?;

        Ok(())
    }

    pub(in crate::function) fn memmove(
        &self,
        destination: PointerValue<'ctx>,
        source: PointerValue<'ctx>,
        length: IntValue<'ctx>,
    ) -> Result<(), CodeGenerationError> {
        self.b().build_memmove(destination, 1, source, 1, length)?;

        Ok(())
    }

    pub(in crate::function) fn memset(
        &self,
        destination: PointerValue<'ctx>,
        value: IntValue<'ctx>,
        length: IntValue<'ctx>,
    ) -> Result<(), CodeGenerationError> {
        self.b().build_memset(destination, 1, value, length)?;

        Ok(())
    }
}
