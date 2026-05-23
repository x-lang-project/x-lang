use std::fmt::Display;

use crate::CompilationFailure;
use crate::Error;
use crate::LinkDiagnostic;

pub fn failed_to_read_object_value_error(object_path: impl Display, error: impl Display) -> String {
    Error::new(format!("Failed to read object {object_path}: {error}")).render_detail()
}

pub fn object_input_is_not_a_file_value(object_path: impl Display) -> String {
    Error::new(format!("Object input is not a file: {object_path}")).render_detail()
}

pub fn object_file_is_empty_value(object_path: impl Display) -> String {
    Error::new(format!("Object file is empty: {object_path}")).render_detail()
}

pub fn failed_to_resolve_object_value_error(
    object_path: impl Display,
    error: impl Display,
) -> String {
    Error::new(format!("Failed to resolve object {object_path}: {error}")).render_detail()
}

pub fn failed_to_run_clang_linker_e(error: impl Display) -> String {
    Error::new(format!("Failed to run clang linker: {error}")).render_detail()
}

pub fn linker_succeeded_without_producing_output_exe(output_exe: impl Display) -> String {
    Error::new(format!("linker succeeded without producing {output_exe}")).render_detail()
}

pub fn linking_failed_with_status_status(status: impl Display) -> String {
    Error::new(format!("Linking failed with status {status}")).render_detail()
}

pub fn linking_failed_no_valid_object_inputs() -> String {
    Error::new("Linking failed: no valid object inputs").render_detail()
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

pub fn object_read(object_path: impl Display, error: impl Display) -> CompilationFailure {
    LinkDiagnostic::ObjectRead.into_failure(failed_to_read_object_value_error(object_path, error))
}

pub fn object_not_file(object_path: impl Display) -> CompilationFailure {
    LinkDiagnostic::ObjectNotFile.into_failure(object_input_is_not_a_file_value(object_path))
}

pub fn object_empty(object_path: impl Display) -> CompilationFailure {
    LinkDiagnostic::ObjectEmpty.into_failure(object_file_is_empty_value(object_path))
}

pub fn object_resolve(object_path: impl Display, error: impl Display) -> CompilationFailure {
    LinkDiagnostic::ObjectResolve
        .into_failure(failed_to_resolve_object_value_error(object_path, error))
}

pub fn clang_launch(error: impl Display) -> CompilationFailure {
    LinkDiagnostic::ClangLaunch.into_failure(failed_to_run_clang_linker_e(error))
}

pub fn output_missing(output_exe: impl Display) -> CompilationFailure {
    LinkDiagnostic::OutputMissing
        .into_failure(linker_succeeded_without_producing_output_exe(output_exe))
}

pub fn clang_failure(status: impl Display) -> CompilationFailure {
    LinkDiagnostic::ClangFailure.into_failure(linking_failed_with_status_status(status))
}

pub fn object_inputs_missing() -> CompilationFailure {
    LinkDiagnostic::ObjectInputsMissing.into_failure(linking_failed_no_valid_object_inputs())
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
