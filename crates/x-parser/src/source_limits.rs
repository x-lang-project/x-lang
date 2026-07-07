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

pub fn validate_source(input: &str) -> Result<(), SourceLimitError> {
    if input.len() > MAX_SOURCE_BYTES {
        return Err(SourceLimitError::SourceTooLarge {
            actual: input.len(),
            maximum: MAX_SOURCE_BYTES,
        });
    }

    SourceScanner::new(input).validate()
}

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
    fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            position: 0,
            nesting_depth: 0,
            token_count: 0,
            prefix_chain: 0,
            delimiter_prefix: 0,
        }
    }

    fn validate(mut self) -> Result<(), SourceLimitError> {
        while self.position < self.bytes.len() {
            let Some(&byte) = self.bytes.get(self.position) else {
                return Err(SourceLimitError::ScannerDidNotAdvance {
                    offset: self.position,
                });
            };

            if self.is_line_comment() {
                self.skip_comment()?;
            } else if matches!(byte, b'"' | b'\'') {
                self.scan_literal()?;
            } else if byte.is_ascii_digit() {
                self.scan_number()?;
            } else if byte.is_ascii_alphabetic() || byte == b'_' {
                self.scan_identifier()?;
            } else {
                self.scan_punctuation(byte)?;
            }
        }

        Ok(())
    }

    fn is_line_comment(&self) -> bool {
        self.bytes
            .get(self.position..)
            .is_some_and(|tail| tail.starts_with(b"//"))
    }

    fn skip_comment(&mut self) -> Result<(), SourceLimitError> {
        let next = scan_line_comment(self.bytes, self.position);
        require_scanner_progress(self.position, next)?;
        self.position = next;
        self.delimiter_prefix = 0;

        Ok(())
    }

    fn advance_token(&mut self, start: usize, next: usize) -> Result<(), SourceLimitError> {
        require_scanner_progress(start, next)?;
        self.position = next;
        record_token(&mut self.token_count, start, MAX_TOKEN_COUNT)?;
        self.prefix_chain = 0;

        Ok(())
    }

    fn scan_literal(&mut self) -> Result<(), SourceLimitError> {
        let start = self.position;
        let next = scan_quoted_literal(self.bytes, start)?;
        self.advance_token(start, next)?;
        self.delimiter_prefix = 0;

        Ok(())
    }

    fn scan_number(&mut self) -> Result<(), SourceLimitError> {
        let start = self.position;
        let next = scan_numeric_token(self.bytes, start)?;
        self.advance_token(start, next)?;
        let literal = self
            .input
            .get(start..self.position)
            .ok_or(SourceLimitError::ScannerDidNotAdvance { offset: start })?;
        validate_numeric_literal(literal, start)?;
        self.delimiter_prefix = 0;

        Ok(())
    }

    fn scan_identifier(&mut self) -> Result<(), SourceLimitError> {
        let start = self.position;
        let next = scan_identifier_token(self.bytes, start)?;
        self.advance_token(start, next)?;
        self.delimiter_prefix = 0;

        Ok(())
    }

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
