//! Lexical analysis (tokenizer)
use crate::tokens::{KeywordKind, Span, Token, TokenKind};

use itertools::{multipeek, MultiPeek};
use std::{
    error, fmt,
    iter::Iterator,
    slice::SliceIndex,
    str::{CharIndices, FromStr},
};

pub fn debug_print_lexer(lexer: Lexer) {
    let source = lexer.source.original;
    println!("Source Byte Count: {}", source.len());

    for result in lexer {
        match result {
            Ok(token) => {
                if token.kind == TokenKind::EOS {
                    println!("{:4}-{} {:?}", token.span.start, token.span.end, token.kind);
                    break;
                } else {
                    let fragment = match &source[token.span.start..=token.span.end] {
                        "\n" => "\\n",
                        "\t" => "\\t",
                        fragment => fragment,
                    };
                    println!(
                        "{:4}-{} {:<10} {:?}",
                        token.span.start, token.span.end, fragment, token.kind
                    );
                }
            }
            Err(err) => println!("{:?}", err),
        }
    }
}

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

/// Lexical analyzer.
pub struct Lexer<'a> {
    source: SourceText<'a>,
    token_start: SourcePos,
}

impl<'a> Lexer<'a> {
    pub fn new(source_code: &'a str) -> Self {
        Self {
            source: SourceText::new(source_code),
            token_start: SourcePos {
                position: 0,
                line: 1,
                column: 1,
            },
        }
    }

    #[rustfmt::skip]
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        use TokenKind as T;

        while !self.source.at_end() {
            if let Some((_, next_char)) = self.source.next_char() {
                self.start_token();

                match next_char {
                    '='               => return Ok(self.make_token(T::Eq)),
                    ':'               => return Ok(self.make_token(T::Colon)),
                    ';'               => return Ok(self.make_token(T::Semicolon)),
                    ' ' | '\t' | '\r' => self.consume_whitespace(),
                    '\n'              => return Ok(self.make_token(T::Newline)),
                    '/'               => {
                        return match self.source.peek_char2() {
                            (Some('/'), Some('/')) => {
                                self.source.next_char();
                                self.source.next_char();
                                Ok(self.consume_until_newline(T::DocComment))
                            }
                            (Some('/'), _) => {
                                self.source.next_char();
                                Ok(self.consume_until_newline(T::Comment))
                            }
                            _ => Ok(self.make_token(T::Slash)),
                        };
                    }
                    '0'..='9'         => return Ok(self.consume_number()),
                    '_' | 'a'..='z'
                        | 'A'..='Z'   => return Ok(self.consume_ident()),
                    _                 => return Err(LexError::UnknownCharacter(next_char)),
                }
            } else {
                // Give end-of-source its own character position.
                self.start_token();
                break;
            }
        }

        Ok(self.make_token(T::EOS))
    }

    /// Prime the lexer state for recording a new token.
    fn start_token(&mut self) {
        self.token_start = SourcePos {
            position: self.source.current.0,
            column: self.source.current_column,
            line: self.source.current_line,
        };
        // println!("{:?} {:?}", self.token_start, self.source.current);
    }

    fn make_token(&mut self, token_kind: TokenKind) -> Token {
        let token_end = SourcePos {
            position: self.source.current.0,
            column: self.source.current_column,
            line: self.source.current_line,
        };

        // Build span.
        let span = Span {
            start: self.token_start.position,
            end: token_end.position,
            start_column: self.token_start.column,
            end_column: token_end.column,
            start_line: self.token_start.line,
            end_line: token_end.line,
        };

        Token {
            kind: token_kind,
            span,
        }
    }

    /// Consume whitespace characters like space, tab and carriage return,
    /// until a non-whitespace character is encountered.
    ///
    /// Newline (\n) is not included because it's tokenized for syntax trivia.
    fn consume_whitespace(&mut self) {
        while let Some((_, ' ')) | Some((_, '\t')) | Some((_, '\r')) = self.source.peek_char() {
            self.source.next_char();
        }
    }

    fn consume_number(&mut self) -> Token {
        self.source.reset_peek();

        // TODO: Hexadecimal numerals
        // TODO: Binary numerals
        // TODO: Octal numerals
        // Consume tokens of the number literal until we
        // encounter a character that's invalid for any
        // of the supported numeral notations.
        while let Some((_, '0'..='9')) = self.source.peek_char() {
            self.source.next_char();
        }

        self.make_token(TokenKind::Number)
    }

    fn consume_ident(&mut self) -> Token {
        self.source.reset_peek();

        while let Some((_, c)) = self.source.peek_char() {
            match c {
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    self.source.next_char();
                }
                _ => break,
            }
        }

        // If a valid keyword can be parsed from the source fragment, then
        // the token is a reserved keyword instead of a user defined identifier.
        let token_kind = KeywordKind::from_str(self.token_fragment())
            .map(TokenKind::Keyword)
            .unwrap_or_else(|_| TokenKind::Ident);
        self.make_token(token_kind)
    }

    fn consume_until_newline(&mut self, token_kind: TokenKind) -> Token {
        while let Some((_, c)) = self.source.peek_char() {
            match c {
                '\n' | '\r' => break,
                _ => {
                    self.source.next_char();
                }
            }
        }

        self.make_token(token_kind)
    }

    fn token_fragment(&self) -> &str {
        &self.source.original[self.token_start.position..=self.source.current.0]
    }
}

/// Implement `Lexer` as an interator for consuming
/// tokens lazily.
impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: return None when tokens are done.
        Some(self.next_token())
    }
}

/// Wrapper for source code that keeps a cursor position.
///
/// Allows forward lookup via peeking.
struct SourceText<'a> {
    /// Keep reference to the source so the parser can
    /// slice fragments from it.
    original: &'a str,

    /// Iterator over UTF-8 encoded source code.
    ///
    /// The `MultiPeek` wrapper allows for arbitrary lookahead by consuming
    /// the iterator internally and buffering the result. This is required
    /// because UTF-8 characters are variable in width. Indexing the string
    /// for individual bytes is possible, but impossible for encoded characters.
    ///
    /// An important semantic feature of `MultiPeek` is that peeking advances
    /// the internal peek cursor by 1. Each call will return the next element.
    /// The peek cursor offset is restored to 0 when calling `MultiPeek::next()`
    /// or `MultiPeek::reset_peek()`.
    source: MultiPeek<CharIndices<'a>>,

    /// Byte position in the source string of the current character.
    current: (usize, char),
    current_line: usize,
    current_column: usize,
}

impl<'a> SourceText<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            original: source,
            source: multipeek(source.char_indices()),
            current: (0, '\0'),
            current_line: 1,
            current_column: 1,
        }
    }

    /// number of bytes in source.
    fn byte_count(&self) -> usize {
        self.original.len()
    }

    /// Advance the cursor and return the next position and character.
    fn next_char(&mut self) -> Option<(usize, char)> {
        if let Some((index, c)) = self.source.next() {
            if c == '\n' {
                self.current_column += 1;
                self.current_line += 1;
            } else {
                self.current_column += 1;
            }
            self.current = (index, c);
            Some((index, c))
        } else {
            // Source code iterator has reached end-of-file.
            //
            // Set the current index to the size of the source
            // string. There is no End-of-file character, so
            // we just set it to the null-byte.
            self.current = (self.byte_count(), '\0');
            None
        }
    }

    /// Peeks the current character in the stream.
    ///
    /// This call advances the peek cursor. Subsequent
    /// calls will look ahead by one character each call.
    fn peek_char(&mut self) -> Option<(usize, char)> {
        self.source.peek().cloned()
    }

    /// Two character lookahead.
    ///
    /// This call advances the peek cursor. Subsequent
    /// calls will look ahead by one character each call.
    fn peek_char2(&mut self) -> (Option<char>, Option<char>) {
        (
            self.source.peek().map(|(_, c)| c).cloned(),
            self.source.peek().map(|(_, c)| c).cloned(),
        )
    }

    /// Reset the stream peek cursor.
    fn reset_peek(&mut self) {
        self.source.reset_peek()
    }

    /// Indicates if the cursor is at the end of the source.
    fn at_end(&self) -> bool {
        self.current.0 >= self.byte_count()
    }
}

#[derive(Debug, Default)]
struct SourcePos {
    position: usize,
    column: usize,
    line: usize,
}

#[derive(Debug, Clone)]
pub enum LexError {
    UnknownCharacter(char),
}

impl error::Error for LexError {}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lexical error")
    }
}
