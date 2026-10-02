use inkwell::attributes::AttributeLoc;
use inkwell::context::Context;
use inkwell::targets::{TargetData, TargetMachine, TargetTriple};

use crate::{CHERI_RISCV64_DATALAYOUT, CHERI_RISCV64_PURECAP_PROFILE, CHERI_RISCV64_TRIPLE};
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

impl CompilationTarget {
    pub(super) fn new(requested: Option<&str>) -> Self {
        let profile = if requested == Some(CHERI_RISCV64_PURECAP_PROFILE) {
            TargetProfile::CheriPurecap
        } else {
            TargetProfile::Native
        };

        let effective_triple = match profile {
            TargetProfile::CheriPurecap => Some(CHERI_RISCV64_TRIPLE),
            TargetProfile::Native => requested,
        };

        Self {
            triple: effective_triple
                .map(TargetTriple::create)
                .unwrap_or_else(TargetMachine::get_default_triple),
            profile,
        }
    }
}
