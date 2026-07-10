use x_ast::Layout;
use x_diagnostics::parser_messages::ParserIssue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FunctionAttribute {
    Memoise,
    NoMangle,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum StructAttribute {
    Layout(Layout),
    Derive(Vec<String>),
}

fn content(text: &str) -> Result<&str, ParserIssue> {
    text.trim()
        .strip_prefix("#[")
        .and_then(|tail| tail.strip_suffix(']'))
        .map(str::trim)
        .ok_or(ParserIssue::MalformedAttribute)
}

pub(crate) fn function_attribute(text: &str) -> Result<FunctionAttribute, ParserIssue> {
    match content(text)? {
        "memoise" => Ok(FunctionAttribute::Memoise),
        "no_mangle" => Ok(FunctionAttribute::NoMangle),
        _ => Err(ParserIssue::UnsupportedFunctionAttribute),
    }
}

pub(crate) fn struct_attribute(text: &str) -> Result<StructAttribute, ParserIssue> {
    let text = content(text)?;
    let (name, arguments) = text
        .split_once('(')
        .and_then(|(name, tail)| {
            tail.strip_suffix(')')
                .map(|current_arguments| (name.trim(), current_arguments.trim()))
        })
        .ok_or(ParserIssue::InvalidStructAttributeSyntax)?;

    match name {
        "layout" => match arguments.to_ascii_lowercase().as_str() {
            "aos" => Ok(StructAttribute::Layout(Layout::AoS)),
            "soa" => Ok(StructAttribute::Layout(Layout::SoA)),
            "auto" => Ok(StructAttribute::Layout(Layout::Auto)),
            _ => Err(ParserIssue::UnsupportedLayout),
        },
        "derive" => {
            let names: Vec<_> = arguments.split(',').map(str::trim).collect();

            if names.iter().all(|name| {
                matches!(
                    *name,
                    "Debug" | "Clone" | "Default" | "Eq" | "PartialEq" | "Hash"
                )
            }) {
                Ok(StructAttribute::Derive(
                    names.into_iter().map(str::to_owned).collect(),
                ))
            } else {
                Err(ParserIssue::UnsupportedDerive)
            }
        }
        _ => Err(ParserIssue::UnsupportedStructAttribute),
    }
}
