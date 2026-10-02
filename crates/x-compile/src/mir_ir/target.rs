use inkwell::attributes::AttributeLoc;
use inkwell::context::Context;
use inkwell::targets::{TargetData, TargetMachine, TargetTriple};

use crate::CHERI_RISCV64_PURECAP_PROFILE;
use crate::CHERI_RISCV64_TRIPLE;
use x_codegen::{CodeGeneration, CodeGenerationOptions, initialize_target_for_triple};
use x_diagnostics::{CompilationFailure, StageDiagnostic};

#[derive(Clone, Copy, PartialEq, Eq)]
enum TargetProfile {
    Native,
    CheriPurecap,
}

pub(super) struct CompilationTarget {
    triple: TargetTriple,
    profile: TargetProfile,
}
