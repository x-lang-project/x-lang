use crate::formatter;

pub(crate) fn compose(inputs: &[(String, String)]) -> Result<Vec<(String, String)>, String> {
    inputs
        .iter()
        .map(|(path, text)| {
            formatter::format_text(text)
                .map(|formatted| (path.clone(), formatted))
                .map_err(|error| format!("{path}: {error}"))
        })
        .collect()
}
