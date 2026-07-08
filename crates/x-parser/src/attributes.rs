use x_ast::Layout;
use x_diagnostics::parser_messages::ParserIssue;

fn content(text: &str) -> Result<&str, ParserIssue> {
    text.trim()
        .strip_prefix("#[")
        .and_then(|tail| tail.strip_suffix(']'))
        .map(str::trim)
        .ok_or(ParserIssue::MalformedAttribute)
}
