use super::CodeGeneration;
use crate::codegen::items::MirProgramItems;
use crate::errors::CodeGenerationError;
use x_mir::MirProgram;

impl CodeGeneration<'_> {
    pub(super) fn declare_struct_types(
        &mut self,
        mir_program: &MirProgram,
    ) -> Result<(), CodeGenerationError> {
        for definition in mir_program.structs() {
            self.records.register(definition);

            if self.module.get_struct_type(&definition.name).is_none() {
                self.context.opaque_struct_type(&definition.name);
            }
        }

        for definition in mir_program.structs() {
            let struct_type = self
                .module
                .get_struct_type(&definition.name)
                .ok_or_else(|| CodeGenerationError::TypeNotFound(definition.name.clone()))?;

            if !struct_type.is_opaque() {
                continue;
            }

            let field_types = definition
                .fields
                .iter()
                .map(|(_, field_type)| self.resolve_llvm_type(field_type))
                .collect::<Result<Vec<_>, _>>()?;

            struct_type.set_body(&field_types, false);
        }

        Ok(())
    }
}
