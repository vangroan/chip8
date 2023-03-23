//! Result and errors.
use std::{
    fmt::{self, Display, Formatter},
    io,
    num::ParseIntError,
    string::FromUtf8Error,
};

use crate::asm::{Span, TokenKind};

pub type Chip8Result<T> = std::result::Result<T, Chip8Error>;

#[derive(Debug)]
pub enum Chip8Error {
    /// VM error during interpreter loop.
    Runtime(&'static str),
    /// Attempt to load a bytecode program that can't fit in memory.
    LargeProgram,
    Asm(AsmError),
    NumberParse(ParseIntError),
    Token(TokenError),
    EOF,
    Fmt(fmt::Error),
    Io(io::Error),
    Utf8(FromUtf8Error),
    Multi(Vec<Chip8Error>),
}

impl Display for Chip8Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Runtime(msg) => write!(f, "runtime error: {}", msg),
            Self::LargeProgram => write!(f, "program too large for VM memory"),
            Self::Asm(err) => write!(f, "parser error: {}", err),
            Self::NumberParse(err) => write!(f, "failed to parse number literal: {err}"),
            Self::Token(err) => write!(f, "token error: {}", err),
            Self::EOF => write!(f, "unexpected end-of-file"),
            Self::Fmt(err) => write!(f, "{}", err),
            Self::Io(err) => write!(f, "{}", err),
            Self::Utf8(err) => write!(f, "{}", err),
            Self::Multi(errors) => {
                // Print all errors separated with a newline
                let count = errors.len();
                for (index, err) in errors.iter().enumerate() {
                    write!(f, "{}", err)?;
                    if index < count {
                        write!(f, "\n")?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Chip8Error {}

impl From<fmt::Error> for Chip8Error {
    fn from(err: fmt::Error) -> Self {
        Chip8Error::Fmt(err)
    }
}

impl From<io::Error> for Chip8Error {
    fn from(err: io::Error) -> Self {
        Chip8Error::Io(err)
    }
}

impl From<FromUtf8Error> for Chip8Error {
    fn from(err: FromUtf8Error) -> Self {
        Chip8Error::Utf8(err)
    }
}

#[derive(Debug)]
pub struct AsmError {
    pub span: Span,
    pub line: String,
    pub line_span: Span,
    pub line_no: usize,
    pub message: String,
}

impl AsmError {
    const MARKER: u8 = 0x5E; // caret (^)
    const SPACE: u8 = 0x20; // space ( )

    pub fn new(source_code: impl AsRef<str>, span: Span, message: impl ToString) -> Self {
        let (line, line_span) = span.surrounding_line(source_code.as_ref());

        // Line numbers start at 1
        let line_no = 1 + Self::count_lines(source_code.as_ref(), span.index as usize);

        Self {
            span,
            line: line.to_string(),
            line_span,
            line_no,
            message: message.to_string(),
        }
    }

    pub fn count_lines(text: &str, index: usize) -> usize {
        let mut count = 0;

        for (i, c) in text.char_indices() {
            if i >= index {
                break;
            }

            if c == '\n' {
                count += 1;
            }
        }

        count
    }
}

impl Display for AsmError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "{}", self.message)?;

        let lineno = format!("{:3}", self.line_no);
        let margin = String::from_utf8(vec![Self::SPACE; lineno.len()]).unwrap_or_default();
        writeln!(f, "{} |", margin)?;

        writeln!(f, "{} | {}", lineno, self.line.trim_end())?;
        //           ^^ margin
        //               ^ padding

        const PADDING: usize = 1;
        let relative_index = (self.span.index - self.line_span.index) as usize;
        // println!("relative index: {}", relative_index);
        let indent =
            String::from_utf8(vec![Self::SPACE; PADDING + relative_index]).unwrap_or_default();

        // EOF span has size 0, so we clamp to 1 for a minimal marker to show up.
        let marker_width = usize::max(1, self.span.size as usize);
        let marker = String::from_utf8(vec![Self::MARKER; marker_width]).unwrap_or_default();
        writeln!(f, "{} |{}{}", margin, indent, marker)?;
        writeln!(f, "{} |", margin)?;

        Ok(())
    }
}

impl From<AsmError> for Chip8Error {
    fn from(err: AsmError) -> Self {
        Chip8Error::Asm(err)
    }
}

/// Error returned when an unexpected token type is encountered.
#[derive(Debug)]
pub struct TokenError {
    pub expected: TokenKind,
    pub encountered: TokenKind,
}

impl std::error::Error for TokenError {}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "encountered token '{:?}', expected '{:?}'",
            self.encountered, self.expected
        )
    }
}

impl From<TokenError> for Chip8Error {
    fn from(err: TokenError) -> Self {
        Chip8Error::Token(err)
    }
}
