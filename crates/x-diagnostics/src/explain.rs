use catalog::explanations;

mod catalog;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorExplanation {
    pub code: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub facts: &'static [&'static str],
    pub fixes: &'static [&'static str],
}

#[derive(Clone, Copy)]
enum ExplanationRender {
    Standard,
    UnknownIdentifier,
}

#[derive(Clone, Copy)]
struct ExplanationEntry {
    detail: ErrorExplanation,
    render: ExplanationRender,
}

impl ExplanationEntry {
    const fn standard(detail: ErrorExplanation) -> Self {
        Self {
            detail,
            render: ExplanationRender::Standard,
        }
    }

    const fn unknown_identifier(detail: ErrorExplanation) -> Self {
        Self {
            detail,
            render: ExplanationRender::UnknownIdentifier,
        }
    }
}

fn explanation_entry(code: &str) -> Option<&'static ExplanationEntry> {
    explanations()
        .iter()
        .find(|entry| entry.detail.code.eq_ignore_ascii_case(code))
}

fn render_unknown_identifier_explanation() -> String {
    [
        "E0006: unknown identifier",
        "",
        "A name can only be used after the compiler can resolve it to a visible",
        "declaration, function, import, or global.",
        "",
        "Broken:",
        "",
        "    print(y)",
        "          ^ `y` has not been declared",
        "",
        "Fixed:",
        "",
        "    let y = 10",
        "    print(y)",
        "",
        "Check these declarations:",
        "",
        "    - local variables",
        "    - function parameters",
        "    - functions in the current module",
        "    - imported names",
        "    - module globals",
        "",
        "Common causes:",
        "",
        "    - the variable was never declared",
        "    - the name is misspelled",
        "    - the value is declared in another module but not imported",
        "    - the value is declared after it is used",
    ]
    .join("\n")
}
