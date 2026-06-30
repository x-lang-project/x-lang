pub const MAX_AST_EXPRESSION_DEPTH: usize = 16;
pub const MAX_AST_STATEMENT_DEPTH: usize = 32;
pub const MAX_AST_NODES: usize = 250_000;

pub use x_diagnostics::frontend_messages::AstValidationError;

fn diagnostic_path(detailed: bool, build: impl FnOnce() -> String) -> String {
    if detailed { build() } else { String::new() }
}

fn require_name(name: &str, path: String) -> Result<(), AstValidationError> {
    if name.trim().is_empty() {
        Err(AstValidationError::EmptySymbol { path })
    } else {
        Ok(())
    }
}
