use super::*;

pub(in crate::clarity) fn analyze(source: &crate::Source, findings: &mut Vec<crate::Finding>) {
    let mut writer = Documentation { source, findings };
    writer.visit_file(&source.ast);
    let mut block = String::new();
    let mut start = 0;
    let mut offset = 0;

    for line in source.text.lines().chain(std::iter::once("")) {
        let type_value = line.trim_start();

        if type_value.starts_with("//")
            && !type_value.starts_with("///")
            && !type_value.starts_with("//!")
        {
            if block.is_empty() {
                start = offset;
            }

            block.push_str(type_value.trim_start_matches("//"));
            block.push('\n');
        } else if !block.is_empty() {
            if (block.contains(';') || block.contains("fn "))
                && (syn::parse_file(&block).is_ok()
                    || syn::parse_str::<syn::Block>(&format!("{{{block}}}")).is_ok())
            {
                source.emit(
                    writer.findings,
                    "commented-out-rust-block",
                    start,
                    "Comment block parses as Rust code.",
                );
            }

            block.clear();
        }
        offset += line.len() + 1;
    }

    let rx = regex::Regex::new(r"\[(?:[^\]]+)\]\(crate::([A-Za-z_][A-Za-z_0-9]*)\)").unwrap();
    let names: std::collections::HashSet<_> = source
        .ast
        .items
        .iter()
        .filter_map(|index| match index {
            syn::Item::Fn(value) => Some(value.sig.ident.to_string()),
            syn::Item::Struct(value) => Some(value.ident.to_string()),
            syn::Item::Enum(value) => Some(value.ident.to_string()),
            syn::Item::Mod(value) => Some(value.ident.to_string()),
            syn::Item::Type(value) => Some(value.ident.to_string()),
            syn::Item::Const(value) => Some(value.ident.to_string()),
            _ => None,
        })
        .collect();

    for character in rx.captures_iter(&source.text) {
        if !names.contains(&character[1]) {
            source.emit(writer.findings,"orphaned-item-reference",character.get(0).unwrap().start(),format!("Local doc target {} is absent from this file's item inventory; module resolution is incomplete.",&character[1]));
        }
    }

    let functions: Vec<_> = source
        .ast
        .items
        .iter()
        .filter_map(|index| {
            if let syn::Item::Fn(function) = index {
                Some(function)
            } else {
                None
            }
        })
        .collect();

    for (index, first_value) in functions.iter().enumerate() {
        for second_value in functions.iter().skip(index + 1) {
            let da = docs(&first_value.attrs);
            let database = docs(&second_value.attrs);
            let wa: Vec<_> = da.split_whitespace().collect();
            let wb: Vec<_> = database.split_whitespace().collect();

            if wa.len() >= 6
                && wa.len() == wb.len()
                && wa != wb
                && wa
                    .iter()
                    .zip(&wb)
                    .filter(|(first_value, second_value)| first_value != second_value)
                    .count()
                    <= 2
                && tokens(&first_value.block) == tokens(&second_value.block)
            {
                crate::ReviewSink {
                    source,
                    findings: writer.findings,
                }
                .at_span(
                    "duplicate-doc-contract-divergence",
                    second_value.sig.ident.span(),
                    "Same-body functions have nearly identical docs with one or two changed words.",
                );
            }
        }
    }
}
