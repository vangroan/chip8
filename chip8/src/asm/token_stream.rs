//! Peekable token stream.
use std::{iter::Peekable, slice::SliceIndex};

use crate::error::{Chip8Error, Chip8Result, TokenError};

use super::{lexer::LexerIter, Lexer, Span, Token, TokenKind};

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
    lexer: Peekable<LexerIter<'a>>,
    /// Keep reference to the source so the parser can
    /// slice fragments from it.
    original: &'a str,
    /// A copy of the previous token.
    /// This can be used to build errors that refer
    /// to the end of the previous token's span.
    prev: Option<Token>,
}

impl<'a> TokenStream<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            original: lexer.source_code(),
            lexer: lexer.into_iter().peekable(),
            prev: None,
        }
    }

    pub fn source_code(&self) -> &str {
        self.original
    }

    pub fn previous_token(&self) -> Option<&Token> {
        self.prev.as_ref()
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
        self.original.get(index)
    }

    /// Helper function to extract the span's string fragment
    /// from the original source code.
    #[inline]
    pub fn span_fragment(&self, span: &Span) -> &str {
        span.fragment(self.original)
    }

    /// Consumes the current token regardless of type.
    ///
    /// Returns `None` when the cursor is at the end of the token stream.
    #[inline]
    pub fn next_token(&mut self) -> Option<Token> {
        self.prev = self.lexer.next();
        self.prev.clone()
    }

    /// Consumes the current token if it matches the given token kind.
    ///
    /// Returns true when matched. Returns false when token kinds
    /// do not match, or the token stream is at the end.
    ///
    /// Does not consume the token if the types do not match.
    pub fn match_token(&mut self, token_kind: TokenKind) -> bool {
        // Ensure clean peek state.
        // self.lexer.reset_peek();

        match self.lexer.peek() {
            Some(token) => {
                let is_match = token.kind == token_kind;
                if is_match {
                    let _ = self.next_token(); // discard
                }
                // peek is reset by next()
                is_match
            }
            None => {
                // self.lexer.reset_peek();
                false
            }
        }
    }

    /// Return the current token and advance the cursor.
    ///
    /// The consumed token must match the given token type, otherwise
    /// a parsing error is returned.
    ///
    /// # Errors
    ///
    /// Returns a [`TokenError`] if the token kind doesn't match.
    ///
    /// # Panics
    ///
    /// Panics when at end-of-file.
    pub fn consume(&mut self, token_kind: TokenKind) -> Chip8Result<Token> {
        // Ensure clean peek state.
        // self.lexer.reset_peek();

        // We should not consume the token if the types don't match.
        match self.lexer.peek() {
            Some(token) => {
                if token.kind != token_kind {
                    // TODO: Return parsing error.
                    Err(Chip8Error::from(TokenError {
                        expected: token_kind,
                        encountered: token.kind,
                    }))
                } else {
                    self.next_token().ok_or_else(|| {
                        // TODO: Change from panic to error
                        panic!("unexpected end-of-file");
                    })
                }
            }
            None => {
                // TODO: Change from panic to error
                panic!("unexpected end-of-file");
            }
        }
    }

    pub fn consume_any(&mut self, token_kinds: &[TokenKind]) -> Chip8Result<Token> {
        for token_kind in token_kinds {
            match self.lexer.peek() {
                Some(token) => {
                    if &token.kind != token_kind {
                        continue;
                    } else {
                        return self.next_token().ok_or_else(|| {
                            // TODO: Change from panic to error
                            panic!("unexpected end-of-file");
                        });
                    }
                }
                None => {
                    // TODO: Change from panic to error
                    panic!("unexpected end-of-file");
                }
            }
        }

        let kind_names = token_kinds
            .iter()
            .map(|kind| format!("{:?}", kind))
            .collect::<Vec<_>>();
        panic!("expected one of: {}", kind_names.join(", "))
    }

    /// Consumes one or more tokens while the token's matches given kind.
    pub fn ignore_many(&mut self, kind: TokenKind) {
        // self.lexer.reset_peek();
        if let Some(token) = self.lexer.peek() {
            if token.kind == kind {
                self.next_token();
            } else {
                // self.lexer.reset_peek();
            }
        }
    }

    /// Consumes one or more tokens while the given predicate tests as `true`.
    pub fn ignore_while(&mut self, predicate: impl Fn(TokenKind) -> bool) {
        // self.lexer.reset_peek();
        while let Some(token) = self.lexer.peek() {
            if predicate(token.kind) {
                self.next_token();
            } else {
                // self.lexer.reset_peek();
                return;
            }
        }
    }

    /// Return the current token without advancing the cursor.
    ///
    /// Subsequent calls to peek will look further ahead.
    /// Call [`reset_peek`] for the lookahead to return to the immediate
    /// next token.
    ///
    /// Returns `None` when lexing is done.
    #[inline]
    pub fn peek(&mut self) -> Option<&Token> {
        self.lexer.peek()
    }

    /// Return the current token kind without advancing the cursor.
    #[inline]
    pub fn peek_kind(&mut self) -> Option<TokenKind> {
        self.lexer.peek().map(|token| token.kind)
    }

    // Reset the internal peek cursor.
    // #[inline]
    // pub fn reset_peek(&mut self) {
    //     self.lexer.reset_peek()
    // }
}
