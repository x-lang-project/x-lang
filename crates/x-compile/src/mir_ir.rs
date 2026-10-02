use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::TargetMachine;

use self::target::CompilationTarget;
use crate::IrCompileOptions;
use crate::frontend;
use x_codegen::{CodeGenerationError, ObjectEmissionOptions, OptimizationPolicy};
use x_diagnostics::{CompilationFailure, SourceIdentifier, source_span_for_diagnostic};
use x_mir::MirProgram;

mod target;
