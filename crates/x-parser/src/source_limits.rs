mod tokens;
use std::borrow::Cow;

use tokens::*;

pub const MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_NESTING_DEPTH: usize = 12;
pub const MAX_NUMERIC_LITERAL_BYTES: usize = 256;
pub const MAX_STRING_LITERAL_BYTES: usize = 1024 * 1024;
pub const MAX_TOKEN_COUNT: usize = 250_000;
pub const MAX_TOKEN_BYTES: usize = 64 * 1024;
pub const MAX_PREFIX_CHAIN: usize = 16;
pub const MAX_DELIMITER_PREFIX: usize = 6;

pub use x_diagnostics::parser_messages::SourceLimitError;

struct SourceScanner<'a> {
    input: &'a str,
    bytes: &'a [u8],
    position: usize,
    nesting_depth: usize,
    token_count: usize,
    prefix_chain: usize,
    delimiter_prefix: usize,
}

impl<'a> SourceScanner<'a> {
    fn scan_punctuation(&mut self, byte: u8) -> Result<(), SourceLimitError> {
        if !byte.is_ascii_whitespace() {
            record_token(&mut self.token_count, self.position, MAX_TOKEN_COUNT)?;
            self.track_prefix_chain(byte)?;
        }

        self.track_delimiter(byte)?;
        self.position = self.position.saturating_add(1);

        Ok(())
    }

    fn track_prefix_chain(&mut self, byte: u8) -> Result<(), SourceLimitError> {
        if matches!(byte, b'!' | b'~' | b'&' | b'*' | b'-' | b'+') {
            self.prefix_chain = self.prefix_chain.saturating_add(1);

            if self.prefix_chain > MAX_PREFIX_CHAIN {
                return Err(SourceLimitError::PrefixChainTooDeep {
                    offset: self.position,
                    maximum: MAX_PREFIX_CHAIN,
                });
            }
        } else if !matches!(byte, b':' | b'.') {
            self.prefix_chain = 0;
        }

        Ok(())
    }

    fn track_delimiter(&mut self, byte: u8) -> Result<(), SourceLimitError> {
        match byte {
            b'(' | b'[' | b'{' => {
                self.delimiter_prefix = self.delimiter_prefix.saturating_add(1);
                if self.delimiter_prefix > MAX_DELIMITER_PREFIX {
                    return Err(SourceLimitError::DelimiterPrefixTooDeep {
                        offset: self.position,
                        maximum: MAX_DELIMITER_PREFIX,
                    });
                }

                self.nesting_depth = self.nesting_depth.saturating_add(1);

                if self.nesting_depth > MAX_NESTING_DEPTH {
                    return Err(SourceLimitError::NestingTooDeep {
                        offset: self.position,
                        maximum: MAX_NESTING_DEPTH,
                    });
                }
            }
            b')' | b']' | b'}' => {
                self.delimiter_prefix = 0;
                self.nesting_depth = self.nesting_depth.saturating_sub(1);
            }
            byte if !byte.is_ascii_whitespace() => self.delimiter_prefix = 0,
            _ => {}
        }

        Ok(())
    }
}
