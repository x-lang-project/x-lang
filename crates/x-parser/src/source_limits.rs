mod tokens;
use std::borrow::Cow;

use tokens::*;

pub const MAX_NUMERIC_LITERAL_BYTES: usize = 256;
pub const MAX_STRING_LITERAL_BYTES: usize = 1024 * 1024;

pub const MAX_TOKEN_BYTES: usize = 64 * 1024;

pub use x_diagnostics::parser_messages::SourceLimitError;
