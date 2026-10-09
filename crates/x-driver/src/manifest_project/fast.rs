use std::fs;

use super::{BuildArtifact, ProjectBuildPlan};
use x_compile::{CompilationFailure, MirObjectOptions};
use x_diagnostics::DriverError;
use x_link::tinycc::{self, LinkOutcome};
use x_utils::target::TinyCcWindows;

pub(super) fn try_execute(
    plan: &ProjectBuildPlan,
) -> Option<Result<BuildArtifact, CompilationFailure>> {
    if plan.os_config.is_some() {
        return None;
    }

    let target = plan.target.as_deref().unwrap_or(TinyCcWindows::HOST_TRIPLE);
    let options = plan.link_inputs.options(plan.optimisation_level, target);

    match tinycc::configured(&options) {
        Ok(false) => None,
        Err(error) => Some(Err(error)),
        Ok(true) => match execute_fast(plan) {
            Ok(artifact) => artifact.map(Ok),
            Err(error) => Some(Err(error)),
        },
    }
}

fn execute_fast(plan: &ProjectBuildPlan) -> Result<Option<BuildArtifact>, CompilationFailure> {
    if let Some(parent) = plan
        .final_exe_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|error| DriverError::manifest_output_directory(parent.display(), error))?;
    }

    let mut compilation = x_compile::compile_mir_program_to_object(
        &plan.mir_program,
        &plan.project_root,
        &plan.name,
        MirObjectOptions {
            optimisation_level: plan.optimisation_level,
            target: Some(TinyCcWindows::TRIPLE),
            save_ir: plan.save_ir,
        },
    )?;
    let target = plan.target.as_deref().unwrap_or(TinyCcWindows::HOST_TRIPLE);
    let outcome = tinycc::link_elf(
        &[compilation.path.to_string_lossy().into_owned()],
        &plan.final_exe_path.to_string_lossy(),
        &plan.link_inputs.options(plan.optimisation_level, target),
    );
    let _ = fs::remove_file(&compilation.path);

    match outcome? {
        LinkOutcome::Linked => {
            compilation.target_triple = target.to_owned();

            Ok(Some(BuildArtifact {
                path: plan.final_exe_path.clone(),
                compilation,
                executable: true,
            }))
        }
        LinkOutcome::Unsupported(_) => Ok(None),
    }
}

#[cfg(feature = "c-backend")]
pub(super) fn try_execute_c(
    plan: &ProjectBuildPlan,
) -> Option<Result<crate::BuildOutput, CompilationFailure>> {
    if !crate::build::c::enabled() || plan.os_config.is_some() || plan.save_ir {
        return None;
    }

    let target = plan.target.as_deref().unwrap_or(TinyCcWindows::HOST_TRIPLE);
    let options = plan.link_inputs.options(plan.optimisation_level, target);

    match tinycc::configured(&options) {
        Ok(false) => None,
        Err(error) => Some(Err(error)),
        Ok(true) => match crate::build::c::compile_checked(
            &plan.mir_program,
            &plan.final_exe_path,
            &options,
            vec![],
        ) {
            Err(error) => Some(Err(error)),
            Ok(Some(artifact)) => Some(Ok(crate::BuildOutput::C(artifact))),
            Ok(None) => None,
        },
    }
}
