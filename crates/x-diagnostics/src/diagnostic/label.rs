use crate::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelStyle {
    Primary,
    Secondary,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub span: Span,
    pub style: LabelStyle,
    pub message: Option<String>,
}
impl Label {
    pub fn primary(span: Span, current_message: impl Into<String>) -> Self {
        Self {
            span,
            style: LabelStyle::Primary,
            message: Some(current_message.into()),
        }
    }

    pub fn secondary(span: Span, current_message: impl Into<String>) -> Self {
        Self {
            span,
            style: LabelStyle::Secondary,
            message: Some(current_message.into()),
        }
    }
}
