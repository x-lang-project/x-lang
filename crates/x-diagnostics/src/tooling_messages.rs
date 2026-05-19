use std::fmt;

use crate::Error;

pub struct ToolingError;

impl ToolingError {
    pub fn temporary_names_exhausted() -> crate::Diagnostic {
        Error::new("temporary file names exhausted")
    }

    pub fn temporary_file_closed() -> crate::Diagnostic {
        Error::new("temporary file is closed")
    }

    pub fn unknown_diagnostic_source(source: usize) -> crate::Diagnostic {
        Error::new(format!("diagnostic references unknown source {source}"))
    }

    pub fn unknown_suggestion_source(source: usize) -> crate::Diagnostic {
        Error::new(format!("suggestion references unknown source {source}"))
    }

    pub fn invalid_suggestion_span(path: impl fmt::Display) -> crate::Diagnostic {
        Error::new(format!("suggestion contains an invalid span in {path}"))
    }

    pub fn json_log_subscriber(error: impl fmt::Display) -> String {
        Error::new(format!("failed to install JSON log subscriber: {error}")).render_detail()
    }

    pub fn log_subscriber(error: impl fmt::Display) -> String {
        Error::new(format!("failed to install log subscriber: {error}")).render_detail()
    }
}

mod source_review;
pub use source_review::SourceReviewError;
