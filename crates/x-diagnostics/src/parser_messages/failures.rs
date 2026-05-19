use std::fmt;

use crate::{CompilationFailure, Error, ParserDiagnostic};

pub fn file_module_project_required() -> CompilationFailure {
    ParserDiagnostic::FileModuleProjectRequired.into_failure(
        "file modules require an x.toml project; standalone compilation cannot resolve `mod`",
    )
}

pub fn empty_input() -> CompilationFailure {
    ParserDiagnostic::EmptyInput.into_failure("Empty input")
}

pub fn ast_validation(error: impl fmt::Display) -> CompilationFailure {
    ParserDiagnostic::AstValidation.into_failure(format!("parsed AST validation failed: {error}"))
}

pub fn syntax_error(position: SyntaxPosition, error: impl fmt::Display) -> CompilationFailure {
    let SyntaxPosition { line, column, byte } = position;

    ParserDiagnostic::Syntax
        .into_failure(format!(
            "syntax error at line {line}, column {column} (byte {byte})\n{error}"
        ))
        .with_span(byte, byte.saturating_add(1))
}
#[derive(Debug, Clone, Copy)]
pub struct SyntaxPosition {
    pub line: usize,
    pub column: usize,
    pub byte: usize,
}
pub fn missing_statement_declaration() -> CompilationFailure {
    ParserDiagnostic::StatementDeclarationMissing
        .into_failure("statement is missing its declaration")
}

pub fn missing_module_name() -> CompilationFailure {
    ParserDiagnostic::ModuleNameMissing.into_failure("module declaration has no name")
}

pub fn unexpected_program_rule(rule: impl fmt::Debug) -> CompilationFailure {
    ParserDiagnostic::UnexpectedProgramRule.into_failure(format!(
        "unexpected parser rule `{rule:?}` at program scope"
    ))
}

pub fn parser_worker_failure(error: impl fmt::Display) -> String {
    Error::new(format!("failed to start bounded parser worker: {error}")).render_detail()
}
pub const NON_STRING_PARSER_PANIC: &str = "non-string parser panic";
pub fn parser_ice_identifier(hash: u64) -> String {
    format!("ICE-PARSER-{hash:016X}")
}

pub fn parser_ice(detail: &str, identifier: &str) -> CompilationFailure {
    Error::new(format!("internal parser invariant failed: {detail}"))
        .with_code(identifier)
        .with_note(format!(
            "reduce the input while preserving diagnostic {identifier}"
        ))
        .with_note("report the reduced source, command, `x --version` and diagnostic ID")
        .into_internal_failure("parser")
}
