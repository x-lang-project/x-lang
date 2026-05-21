use std::fmt::Display;

use crate::Error;

pub struct SourceReviewError;
impl SourceReviewError {
    pub fn invalid_root_or_limit() -> String {
        Error::new("invalid root or max_file_bytes").render_detail()
    }

    pub fn non_idempotent_pipeline() -> String {
        Error::new("non-idempotent formatting pipeline").render_detail()
    }

    pub fn rollback_failed(error: impl Display, rollback_errors: impl Display) -> String {
        Error::new(format!("{error}; rollback issues: {}", rollback_errors)).render_detail()
    }

    pub fn rustfmt_stdin_unavailable() -> String {
        Error::new("Rustfmt stdin is unavailable").render_detail()
    }

    pub fn rustfmt_changed_syntax() -> String {
        Error::new("Rustfmt changed syntax beyond layout and trailing separators; output rejected")
            .render_detail()
    }

    pub fn rustfmt_changed_shebang() -> String {
        Error::new("Rustfmt changed the file shebang; output rejected").render_detail()
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

    pub fn unknown_rule(identifier: impl Display) -> String {
        Error::new(format!("unknown rule: {identifier}")).render_detail()
    }
}
