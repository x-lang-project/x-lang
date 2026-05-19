use crate::Severity;

#[derive(Clone, Copy)]
pub(super) enum ColourRole {
    Error,
    Warning,
    Help,
    Note,
    Code,
    Path,
    Line,
    Border,
    Source,
    Dim,
    Insert,
    Replace,
    Reset,
}

impl ColourRole {
    pub(super) fn for_severity(severity: Severity) -> Self {
        match severity {
            Severity::Error => Self::Error,
            Severity::Warning => Self::Warning,
            Severity::Note => Self::Note,
            Severity::Help => Self::Help,
        }
    }

    pub(super) fn ansi(self, enabled: bool) -> &'static str {
        if !enabled {
            return "";
        }

        match self {
            Self::Error => "\x1b[38;5;203m",
            Self::Warning => "\x1b[38;5;215m",
            Self::Help => "\x1b[38;5;114m",
            Self::Note => "\x1b[38;5;117m",
            Self::Code => "\x1b[38;5;141m",
            Self::Path => "\x1b[38;5;110m",
            Self::Line => "\x1b[38;5;245m",
            Self::Border => "\x1b[38;5;240m",
            Self::Source => "\x1b[38;5;252m",
            Self::Dim => "\x1b[38;5;247m",
            Self::Insert => "\x1b[38;5;151m",
            Self::Replace => "\x1b[38;5;229m",
            Self::Reset => "\x1b[0m",
        }
    }
}
