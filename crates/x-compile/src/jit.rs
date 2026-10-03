use std::path::Path;

use inkwell::values::AsValueRef;
use inkwell::{OptimizationLevel, context::Context};

use crate::{IrCompileOptions, frontend, mir_ir};
use x_diagnostics::{CliError, CompilationFailure};
use x_mir::MirProgram;

fn error(message: impl ToString) -> CompilationFailure {
    CliError::operational(message.to_string()).into_failure()
}

impl<'ctx> mir_ir::GeneratedMirIr<'ctx> {
    fn into_module(self) -> Result<inkwell::module::Module<'ctx>, String> {
        self.take_module_for_jit()
            .map_err(|error| error.to_string())
    }
}
