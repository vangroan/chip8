//! Lexical analysis
use crate::asm::tokens::VReg;

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
        let mut cursor = Cursor::new(source_code);

        // Initial state of the cursor is a non-existant EOF char,
        // but the initial state of the lexer should be a valid
        // token starting character.
        //
        // Prime the cursor for the first iteration.
        cursor.next();

        // For what it's worth, the cursor gets to decide what the
        // initial byte position is.
        let start_pos = cursor.offset();

        Self {
            cursor,
            original: source_code,
            start_pos,
        }
    }

    /// Original source code that was passed in during construction.
    pub fn source_code(&self) -> &'a str {
        self.original
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

        // Erase leading whitespace.
        while is_whitespace(self.cursor.current()) {
            self.cursor.next_char();
        }

        // Erase comment line.
        if matches!(self.cursor.current(), ';') {
            self.erase_comment();
        }

        // Assume that lexer initialization, or previous iteration,
        // leaves the cursor at the next character.
        self.start_token();

        match self.cursor.current() {
            ',' => self.make_token(TK::Comma),
            '.' => self.make_token(TK::Dot),
            ':' => self.make_token(TK::Colon),
            ';' => self.make_token(TK::Semicolon),
            '[' => self.make_token(TK::LeftBracket),
            ']' => self.make_token(TK::RightBracket),
            '\r' => {
                // Windows :(
                if self.cursor.peek() == '\n' {
                    self.cursor.next();
                }
                self.make_token(TK::Newline)
            }
            '\n' => self.make_token(TK::Newline),
            '_' | 'a'..='z' | 'A'..='Z' => self.consume_ident(),
            '0'..='9' => self.consume_number(),

            EOF_CHAR => self.make_token(TK::EOF),
            _ => self.make_token(TK::Unknown),
        }
    }

    /// Indicates whether the lexer is at the end of the source.
    ///
    /// Note that source can contain '\0' (end-of-file) characters,
    /// but not be at the actual end. It's thus important to verify
    /// with this function whenever a [`TokenKind::EOF`] is encountered.
    pub fn at_end(&self) -> bool {
        self.cursor.at_end()
    }

    /// Create a span using the starting position of the current token,
    /// and the current offset of the cursor.
    fn make_span(&self) -> Span {
        let start = self.start_pos;
        let end = self.cursor.peek_offset();

        // start and end can be equal, and a token can have 0 size.
        debug_assert!(end >= start);
        let size = end - start;

        Span { index: start, size }
    }

    fn fragment(&self) -> &str {
        self.make_span().fragment(self.original)
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
        // After this token is built, the lexer's internal state
        // is no longer dedicated to this iteration, but to preparing
        // for the next iteration.
        let token = Token {
            span: self.make_span(),
            kind,
        };

        // Position the cursor to the starting character for the
        // next token, so the lexer's internal state is primed
        // for the next iteration.
        self.cursor.next();
        debug_assert_eq!(self.cursor.offset(), token.span.end());

        token
    }
}

/// Specialised tokens.
impl<'a> Lexer<'a> {
    /// Erase comment line up to, but not including, the trailing newline.
    fn erase_comment(&mut self) {
        debug_assert_eq!(self.cursor.current(), ';');

        while !is_newline(self.cursor.current()) {
            self.cursor.next();
        }
    }

    /// Make an identifier token.
    fn consume_ident(&mut self) -> Token {
        debug_assert!(is_letter(self.cursor.current()));

        while is_letter_or_digit(self.cursor.peek()) {
            self.cursor.next();
        }

        // Attempt to convert identifier to keyword, or a register.
        let token_kind = match Keyword::parse(self.fragment()) {
            Some(keyword) => TokenKind::Keyword(keyword),
            None => match VReg::parse(self.fragment()) {
                Some(vregister) => TokenKind::Register(vregister),
                None => TokenKind::Ident,
            },
        };

        self.make_token(token_kind)
    }

    /// Make a number literal token.
    fn consume_number(&mut self) -> Token {
        debug_assert!(is_digit(self.cursor.current()));

        // Number format marker located in second position.
        if self.cursor.current() == '0' && matches!(self.cursor.peek(), 'b' | 'x') {
            self.cursor.next();
        }

        while is_hex_number(self.cursor.peek()) {
            self.cursor.next();
        }

        self.make_token(TokenKind::Number)
    }
}

/// Test whether the character is considered whitespace
/// that should be ignored by the parser later.
///
/// Doesn't include newline characters, because newlines
/// are significant, specifying end-of-statement.
fn is_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0020}' // space
            | '\u{0009}' // tab
            | '\u{00A0}' // no-break space
            | '\u{FEFF}' // zero width no-break space
    )
}

fn is_newline(c: char) -> bool {
    matches!(c, '\r' | '\n')
}

fn is_hex_number(c: char) -> bool {
    is_digit(c) || is_hex_letter(c)
}

#[allow(clippy::manual_is_ascii_check)] // consistency with other functions
fn is_digit(c: char) -> bool {
    matches!(c, '0'..='9')
}

fn is_hex_letter(c: char) -> bool {
    matches!(c, 'a'..='f' | 'A'..='F')
}

fn is_letter(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '_')
}

fn is_letter_or_digit(c: char) -> bool {
    is_letter(c) || is_digit(c)
}

impl<'a> IntoIterator for Lexer<'a> {
    type Item = Token;
    type IntoIter = LexerIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        LexerIter {
            lexer: self,
            done: false,
        }
    }
}

/// Convenience iterator that wraps the lexer.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct LexerIter<'a> {
    // Track end so an EOF token is emitted once.
    done: bool,
    lexer: Lexer<'a>,
}

impl<'a> Iterator for LexerIter<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.lexer.at_end() {
            if self.done {
                None
            } else {
                // Emit that last EOF token
                self.done = true;
                Some(self.lexer.next_token())
            }
        } else {
            Some(self.lexer.next_token())
        }
    }
}
