use std::fmt::Display;

use crate::CompilationFailure;
use crate::Error;
use crate::LinkDiagnostic;

pub fn failed_to_run_clang_linker_e(error: impl Display) -> String {
    Error::new(format!("Failed to run clang linker: {error}")).render_detail()
}

pub fn linking_failed_with_status_status(status: impl Display) -> String {
    Error::new(format!("Linking failed with status {status}")).render_detail()
}

pub fn llvm_clang_was_not_found_set_x_clang_or_llvm_sys_prefix(
    required_version: impl Display,
    prefix_environment: impl Display,
) -> String {
    Error::new(format!("LLVM {required_version} clang was not found; set X_CLANG or {prefix_environment} to the pinned toolchain"))
        .render_detail()
}

pub fn configured_by_selected_value_but_it_could_not_be_executed(
    configured_by: impl Display,
    clang_path: impl Display,
) -> String {
    Error::new(format!(
        "{configured_by} selected {clang_path}, but it could not be executed"
    ))
    .render_detail()
}

pub fn configured_by_selected_value_which_is_not_llvm_required_clang_version_value(
    configured_by: impl Display,
    clang_path: impl Display,
    required_clang_version: impl Display,
    version_output: impl Display,
) -> String {
    Error::new(format!("{configured_by} selected {clang_path}, which is not LLVM {required_clang_version}: {version_output}")).render_detail()
}

pub fn clang_launch(error: impl Display) -> CompilationFailure {
    LinkDiagnostic::ClangLaunch.into_failure(failed_to_run_clang_linker_e(error))
}

pub fn clang_failure(status: impl Display) -> CompilationFailure {
    LinkDiagnostic::ClangFailure.into_failure(linking_failed_with_status_status(status))
}

pub fn clang_missing(
    required_version: impl Display,
    prefix_environment: impl Display,
) -> CompilationFailure {
    LinkDiagnostic::ClangMissing.into_failure(
        llvm_clang_was_not_found_set_x_clang_or_llvm_sys_prefix(
            required_version,
            prefix_environment,
        ),
    )
}

pub fn configured_clang_unavailable(
    configured_by: impl Display,
    clang_path: impl Display,
) -> CompilationFailure {
    LinkDiagnostic::ConfiguredClangUnavailable.into_failure(
        configured_by_selected_value_but_it_could_not_be_executed(configured_by, clang_path),
    )
}

pub fn configured_clang_version(
    configured_by: impl Display,
    clang_path: impl Display,
    required_clang_version: impl Display,
    version_output: impl Display,
) -> CompilationFailure {
    LinkDiagnostic::ConfiguredClangVersion.into_failure(
        configured_by_selected_value_which_is_not_llvm_required_clang_version_value(
            configured_by,
            clang_path,
            required_clang_version,
            version_output,
        ),
    )
}
