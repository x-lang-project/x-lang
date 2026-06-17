use std::thread;

use std::io::Write;
use std::process::{Command, Stdio};

use x_diagnostics::tooling_messages::SourceReviewError;

mod normalize;

pub fn format_text(text: &str) -> Result<String, String> {
    let before = syn::parse_file(text).map_err(SourceReviewError::input_parse)?;
    let mut child = Command::new("rustfmt")
        .args([
            "--emit",
            "stdout",
            "--edition",
            "2024",
            "--config",
            "skip_children=true,reorder_imports=false,reorder_modules=false,newline_style=Unix",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(SourceReviewError::rustfmt_start)?;
    let input = child
        .stdin
        .take()
        .ok_or_else(SourceReviewError::rustfmt_stdin_unavailable)?;
    // Write concurrently so a full output pipe cannot deadlock stdin.
    let owned = text.as_bytes().to_vec();
    let feeder = thread::spawn(move || {
        let mut input = input;

        input.write_all(&owned)
    });
    let result = child
        .wait_with_output()
        .map_err(SourceReviewError::rustfmt_collect)?;
    let input_result = feeder
        .join()
        .map_err(|_| SourceReviewError::rustfmt_writer_panicked())?;

    if !result.status.success() {
        return Err(SourceReviewError::rustfmt_failed(
            result.status,
            String::from_utf8_lossy(&result.stderr).trim(),
        ));
    }

    input_result.map_err(SourceReviewError::rustfmt_supply)?;
    let formatted = String::from_utf8(result.stdout).map_err(SourceReviewError::rustfmt_utf8)?;
    let after = syn::parse_file(&formatted).map_err(SourceReviewError::formatted_parse)?;

    if normalize::fingerprint(before.clone()) != normalize::fingerprint(after.clone()) {
        return Err(SourceReviewError::rustfmt_changed_syntax());
    }

    if before.shebang != after.shebang {
        return Err(SourceReviewError::rustfmt_changed_shebang());
    }

    Ok(formatted)
}

pub fn check(source: &crate::Source, output: &mut Vec<crate::Finding>) -> Result<(), String> {
    let formatted = format_text(&source.text)?;

    if formatted == source.text {
        return Ok(());
    }

    let first = source
        .text
        .split_inclusive('\n')
        .zip(formatted.split_inclusive('\n'))
        .position(|(first_value, second_value)| first_value != second_value)
        .unwrap_or_else(|| source.text.lines().count().min(formatted.lines().count()));
    source.emit(
        output,
        "unformatted-file",
        source.line_offset(first + 1),
        "Rustfmt changes whitespace/layout while preserving normalised Rust tokens.",
    );

    Ok(())
}
