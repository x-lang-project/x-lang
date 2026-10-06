use std::ffi;

use std::path::Path;

use inkwell::values::AsValueRef;
use inkwell::{OptimizationLevel, context::Context};

use crate::{IrCompileOptions, frontend, mir_ir};
use x_diagnostics::{CliError, CompilationFailure};
use x_mir::MirProgram;

fn error(message: impl ToString) -> CompilationFailure {
    CliError::operational(message.to_string()).into_failure()
}

/// # Safety
/// Resolved functions must match their declared C ABI and remain live until execution ends.
pub unsafe fn execute_mir_jit(
    program: &MirProgram,
    optimisation_level: u8,
    mut resolve: impl FnMut(&str) -> Option<usize>,
) -> Result<i32, CompilationFailure> {
    let context = Context::create();
    let generated = mir_ir::generate_mir_ir(
        &context,
        program,
        frontend::SourceInput {
            text: "",
            path: Path::new("<jit>"),
        },
        IrCompileOptions {
            optimisation_level,
            ..Default::default()
        },
    )?;
    let module = generated.into_module().map_err(error)?;
    module.verify().map_err(error)?;
    let main = module
        .get_function("main")
        .ok_or_else(|| error("JIT requires main()"))?;

    if main.count_basic_blocks() == 0
        || main.count_params() != 0
        || main.get_type().is_var_arg()
        || main.get_call_conventions() != 0
    {
        return Err(error("JIT requires a defined main() with the C ABI"));
    }

    let returns_integer = match main.get_type().get_return_type() {
        None => false,
        Some(inkwell::types::BasicTypeEnum::IntType(value_type))
            if value_type.get_bit_width() == 32 =>
        {
            true
        }
        _ => return Err(error("JIT main() must return i32 or void")),
    };

    let mut mappings = Vec::new();

    for function in module.get_functions() {
        if function.count_basic_blocks() != 0
            || unsafe {
                inkwell::llvm_sys::core::LLVMGetFirstUse(function.as_value_ref()).is_null()
            }
        {
            continue;
        }

        let name = function.get_name().to_str().map_err(error)?;

        if name.starts_with("llvm.") {
            continue;
        }

        let address = resolve(name)
            .ok_or_else(|| error(format!("JIT cannot resolve external function {name}")))?;
        mappings.push((function, address));
    }

    let engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .map_err(error)?;

    for (function, address) in mappings {
        engine.add_global_mapping(&function, address);
    }

    for name in ["__chkstk", "memcpy", "memmove", "memset", "fmod", "fmodf"] {
        if let Some(address) = resolve(name) {
            let name = ffi::CString::new(name).map_err(error)?;

            unsafe {
                inkwell::llvm_sys::support::LLVMAddSymbol(
                    name.as_ptr(),
                    address as *mut ffi::c_void,
                );
            }
        }
    }

    engine.run_static_constructors();
    let code = if returns_integer {
        let entry = unsafe { engine.get_function::<unsafe extern "C" fn() -> i32>("main") }
            .map_err(error)?;

        unsafe { entry.call() }
    } else {
        let entry =
            unsafe { engine.get_function::<unsafe extern "C" fn()>("main") }.map_err(error)?;
        unsafe {
            entry.call();
        }

        0
    };

    engine.run_static_destructors();

    Ok(code)
}

impl<'ctx> mir_ir::GeneratedMirIr<'ctx> {
    fn into_module(self) -> Result<inkwell::module::Module<'ctx>, String> {
        self.take_module_for_jit()
            .map_err(|error| error.to_string())
    }
}
