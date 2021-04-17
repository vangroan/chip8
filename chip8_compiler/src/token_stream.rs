//! Buffered stream of tokens for look ahead.
use crate::{
    lex::{LexError, Lexer},
    tokens::{Span, Token, TokenKind},
};

use itertools::{multipeek, MultiPeek};
use std::{error, fmt, iter::Iterator, slice::SliceIndex};

/// Buffered stream of tokens that allows arbitrary look ahead.
///
/// Tokens are lazily lexed. Peeking or consuming the next token
/// triggers the internal lexer.
///
/// The peek semantics are determined by the internal `MultiPeek`.
/// Calling `TokenStream::peek` is not idempotent, advancing a peek
/// cursor forward by one token for each `peek()` call. The cursor
/// can be reset explicitly using `TokenStream::reset_peek` or
/// implicitly by calling one of the consuming methods.
pub struct TokenStream<'a> {
    lexer: MultiPeek<Lexer<'a>>,
    /// Keep reference to the source so the parser can
    /// slice fragments from it.
    source: &'a str,
}

impl<'a> TokenStream<'a> {
    #[inline]
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            source: lexer.source.original,
            lexer: multipeek(lexer),
        }
    }

    /// Slice a fragment of source code.
    ///
    /// Returns `None` if the given index is out
    /// of bounds.
    #[inline]
    pub fn fragment<I>(&self, index: I) -> Option<&str>
    where
        I: SliceIndex<str, Output = str>,
    {
        self.source.get(index)
    }

    #[inline]
    pub fn fragment_span(&self, span: &Span) -> Option<&str> {
        self.fragment(span.start..=span.end)
    }

    /// Consumes the current token regardless of type.
    ///
    /// Returns `None` when the cursor is at the end of the token stream.
    #[inline]
    pub fn next_token(&mut self) -> Option<Result<Token, LexError>> {
        self.lexer.next()
    }

    /// Consumes the current token if it matches the given token type.
    ///
    /// Returns true when matched. Returns false when token types
    /// do not match, or the token stream is at the end.
    ///
    /// Does not consume the token if the types do not match.
    pub fn match_token(&mut self, token_kind: TokenKind) -> bool {
        // Ensure clean peek state.
        self.lexer.reset_peek();

        match self.lexer.peek() {
            Some(Ok(token)) => {
                let is_match = token.kind == token_kind;
                if is_match {
                    self.lexer.next();
                }
                self.lexer.reset_peek();
                is_match
            }
            _ => {
                self.lexer.reset_peek();
                false
            }
        }
    }

    /// Return the current token with advancing the cursor.
    ///
    /// The consumed token must match the given token type, otherwise
    /// a lexical error is returned.
    pub fn consume(&mut self, token_kind: TokenKind) -> Result<Token, TokenError> {
        // Ensure clean peek state.
        self.lexer.reset_peek();

        // We should not consume the token if the types don't match.
        match self.lexer.peek() {
            Some(Ok(token)) => {
                if token.kind != token_kind {
                    // TODO: Return parsing error.
                    Err(TokenError::Mismatch {
                        expected: token_kind,
                        encountered: token.kind,
                    })
                } else {
                    self.lexer
                        .next()
                        .ok_or(TokenError::EndOfSource)?
                        .map_err(TokenError::Lex)
                }
            }
            Some(Err(err)) => Err(TokenError::Lex(err.clone())),
            None => Err(TokenError::EndOfSource),
        }
    }

    /// Consumes one or more new lines until something else is reached.
    #[inline]
    pub fn match_lines(&mut self) {
        self.lexer.reset_peek();
        if let Some(Ok(token)) = self.lexer.peek() {
            if token.kind == TokenKind::Newline {
                self.lexer.next();
            } else {
                self.lexer.reset_peek();
                return;
            }
        }
    }

    /// Return the current token without advancing the cursor.
    ///
    /// Returns `None` when lexing is done.
    #[inline]
    pub fn peek(&mut self) -> Result<&Token, TokenError> {
        match self.lexer.peek() {
            Some(result) => result.as_ref().map_err(|err| TokenError::Lex(err.clone())),
            None => Err(TokenError::EndOfSource),
        }
    }

    /// Set peek cursor back to the current cursor.
    pub fn reset_peek(&mut self) {
        self.lexer.reset_peek()
    }
}

/// Error returned when an unexpected token type is encountered.
#[derive(Debug)]
pub enum TokenError {
    Mismatch {
        expected: TokenKind,
        encountered: TokenKind,
    },
    EndOfSource,
    Lex(LexError),
}

impl error::Error for TokenError {}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TokenError as E;
        match self {
            E::Mismatch {
                expected,
                encountered,
            } => write!(
                f,
                "encountered unexpected token '{}', expected '{}'",
                encountered, expected
            ),
            E::EndOfSource => write!(f, "unexpected end of source code"),
            E::Lex(err) => fmt::Display::fmt(err, f),
        }
    }
}

impl From<LexError> for TokenError {
    fn from(err: LexError) -> Self {
        TokenError::Lex(err)
    }
}
