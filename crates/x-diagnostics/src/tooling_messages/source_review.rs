use std::fmt::Display;

use crate::Error;

pub struct SourceReviewError;
impl SourceReviewError {
    pub fn duplicate_report_paths() -> String {
        Error::new("Primary and text report paths must be distinct").render_detail()
    }

    pub fn missing_source_filename() -> String {
        Error::new("missing source filename").render_detail()
    }

    pub fn invalid_root_or_limit() -> String {
        Error::new("invalid root or max_file_bytes").render_detail()
    }

    pub fn non_idempotent_pipeline() -> String {
        Error::new("non-idempotent formatting pipeline").render_detail()
    }

    pub fn unsafe_source_path(path: impl Display) -> String {
        Error::new(format!("refusing symlink or out-of-root source {}", path)).render_detail()
    }

    pub fn source_too_large(path: impl Display) -> String {
        Error::new(format!("file too large: {}", path)).render_detail()
    }

    pub fn source_read_only(path: impl Display) -> String {
        Error::new(format!("source is read-only: {}", path)).render_detail()
    }

    pub fn source_changed_during_plan(path: impl Display) -> String {
        Error::new(format!("source changed during planning: {}", path)).render_detail()
    }

    pub fn source_changed_before_write(path: impl Display) -> String {
        Error::new(format!("source changed before write: {}", path)).render_detail()
    }

    pub fn concurrent_source_preserved(path: impl Display) -> String {
        Error::new(format!(
            "concurrently changed file not overwritten: {}",
            path
        ))
        .render_detail()
    }

    pub fn rollback_failed(error: impl Display, rollback_errors: impl Display) -> String {
        Error::new(format!("{error}; rollback issues: {}", rollback_errors)).render_detail()
    }

    pub fn rustfmt_stdin_unavailable() -> String {
        Error::new("Rustfmt stdin is unavailable").render_detail()
    }

    pub fn rustfmt_writer_panicked() -> String {
        Error::new("Rustfmt input writer panicked").render_detail()
    }

    pub fn rustfmt_changed_syntax() -> String {
        Error::new("Rustfmt changed syntax beyond layout and trailing separators; output rejected")
            .render_detail()
    }

    pub fn rustfmt_changed_shebang() -> String {
        Error::new("Rustfmt changed the file shebang; output rejected").render_detail()
    }

    pub fn input_parse(error: impl Display) -> String {
        Error::new(format!("Input Rust parse error: {error}")).render_detail()
    }

    pub fn rustfmt_start(error: impl Display) -> String {
        Error::new(format!("Cannot start rustfmt: {error}")).render_detail()
    }

    pub fn rustfmt_collect(error: impl Display) -> String {
        Error::new(format!("Cannot collect rustfmt output: {error}")).render_detail()
    }

    pub fn rustfmt_supply(error: impl Display) -> String {
        Error::new(format!("Cannot supply rustfmt input: {error}")).render_detail()
    }

    pub fn rustfmt_utf8(error: impl Display) -> String {
        Error::new(format!("Rustfmt output is not UTF-8: {error}")).render_detail()
    }

    pub fn formatted_parse(error: impl Display) -> String {
        Error::new(format!("Formatted Rust parse error: {error}")).render_detail()
    }

    pub fn rustfmt_failed(status: impl Display, detail: impl Display) -> String {
        Error::new(format!("Rustfmt failed ({}): {}", status, detail)).render_detail()
    }

    pub fn root_not_directory() -> String {
        Error::new("root must be a directory").render_detail()
    }

    pub fn invalid_file_limit() -> String {
        Error::new("max_file_bytes must be positive").render_detail()
    }

    pub fn source_outside_root() -> String {
        Error::new("symlink or path outside root skipped").render_detail()
    }

    pub fn file_limit_exceeded() -> String {
        Error::new("max_file_bytes exceeded").render_detail()
    }

    pub fn unknown_rule(identifier: impl Display) -> String {
        Error::new(format!("unknown rule: {identifier}")).render_detail()
    }

    pub fn output_in_write_mode() -> String {
        Error::new("--write-format prints its result; --output is only for scans").render_detail()
    }

    pub fn output_parent_missing() -> String {
        Error::new("output needs a parent directory").render_detail()
    }

    pub fn output_inside_source() -> String {
        Error::new("output must be outside scanned tree").render_detail()
    }

    pub fn output_filename_missing() -> String {
        Error::new("missing output filename").render_detail()
    }

    pub fn output_exists() -> String {
        Error::new("output already exists; choose a new filename").render_detail()
    }

    pub fn output_directory_missing(error: impl Display) -> String {
        Error::new(format!("output directory must exist: {error}")).render_detail()
    }
}
