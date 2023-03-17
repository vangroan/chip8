//! Lexical analysis
use super::{
    cursor::{Cursor, EOF_CHAR},
    tokens::{Keyword, Span, Token, TokenKind},
};

pub struct Lexer<'a> {
    /// Character scanner
    cursor: Cursor<'a>,
    /// Keep reference to the source so the parser can
    /// slice fragments from it.
    original: &'a str,
    /// Start absolute byte position of the current token
    /// in the source.
    start_pos: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(source_code: &'a str) -> Self {
        Self {
            cursor: Cursor::new(source_code),
            original: source_code,
            start_pos: 0,
        }
    }

    /// Original source code that was passed in during construction.
    pub fn source_code(&self) -> &str {
        &self.original
    }

    /// Scan the source characters and construct the next token.
    ///
    /// ## Implementation
    ///
    /// The internal iteration of the lexer follows this convention:
    ///
    /// Each iteration (`next_token` call) starts with the assumption that
    /// the internal cursor is pointing to the start of the remaining source
    /// to be consumed.
    ///
    /// Initially, the lexer must be constructed with a cursor pointing to
    /// the start of the source.
    ///
    /// When an iteration is done building a token, it must leave the cursor
    /// at the start of the next token's text. It may not finish leaving the
    /// cursor pointing into its own token.
    pub fn next_token(&mut self) -> Token {
        use TokenKind as TK;

        self.start_token();

        match self.cursor.next_char() {
            Some(',') => self.make_token(TK::Comma),
            Some('.') => self.make_token(TK::Dot),
            Some(':') => self.make_token(TK::Colon),
            Some(';') => self.make_token(TK::Semicolon),
            Some('\n') => {
                if self.cursor.peek() == '\r' {
                    self.cursor.next();
                }
                self.make_token(TK::Newline)
            }
            Some('_' | 'a'..='z' | 'A'..='Z') => self.make_ident(),
            Some('0'..='9') => self.make_number(),

            None | Some(EOF_CHAR) => self.make_token(TK::EOF),
            _ => self.make_token(TK::Unknown),
        }
    }

    /// Primes the lexer to consume the next token.
    fn start_token(&mut self) {
        self.start_pos = self.cursor.offset();
    }

    /// Build a token, using the source text from the position
    /// stored by [`start_token`](struct.Lexer.html#fn-start_token) to the
    /// current cursor position.
    ///
    /// Also prepare the cursor for the next iteration.
    fn make_token(&mut self, kind: TokenKind) -> Token {
        let start = self.start_pos;
        let end = self.cursor.peek_offset();

        // start and end can be equal, and a token can have 0 size.
        debug_assert!(end >= start);
        let size = end - start;

        // After this token is built, the lexer's internal state
        // is no longer dedicated to this iteration, but to preparing
        // for the next iteration.
        let token = Token {
            span: Span { index: start, size },
            kind,
        };

        // Position the cursor to the starting character for the
        // next token, so the lexer's internal state is primed
        // for the next iteration.
        // self.cursor.bump();

        token
    }
}

/// Specialised tokens.
impl<'a> Lexer<'a> {
    /// Make an identifier token.
    fn make_ident(&self) -> Token {
        todo!()
    }

    /// make a number literal token.
    fn make_number(&self) -> Token {
        todo!()
    }
}
