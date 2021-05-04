//! Lexical analysis (tokenizer)
use crate::tokens::{KeywordKind, Span, Token, TokenKind};

use itertools::{multipeek, MultiPeek};
use std::{
    error, fmt,
    iter::Iterator,
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
                    println!("{:4}-{} {:<10} {:?}", token.span.start, token.span.end, fragment, token.kind);
                }
            }
            Err(err) => println!("{:?}", err),
        }
    }
}

/// Lexical analyzer.
pub struct Lexer<'a> {
    pub(crate) source: SourceText<'a>,
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
                    '+'               => return Ok(self.make_token(T::Plus)),
                    '-'               => {
                        if let Some((_, '>')) = self.source.peek_char() {
                            return Ok(self.make_token(T::Arrow))
                        } else {
                            return Ok(self.make_token(T::Minus))
                        }
                    },
                    '*'               => return Ok(self.make_token(T::Star)),
                    '='               => return Ok(self.make_token(T::Eq)),
                    ','               => return Ok(self.make_token(T::Comma)),
                    '.'               => return Ok(self.make_token(T::Dot)),
                    ':'               => return Ok(self.make_token(T::Colon)),
                    ';'               => return Ok(self.make_token(T::Semicolon)),
                    '('               => return Ok(self.make_token(T::LeftParen)),
                    ')'               => return Ok(self.make_token(T::RightParen)),
                    '{'               => return Ok(self.make_token(T::LeftBrace)),
                    '}'               => return Ok(self.make_token(T::RightBrace)),
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

        Token { kind: token_kind, span }
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
pub(crate) struct SourceText<'a> {
    /// Keep reference to the source so the parser can
    /// slice fragments from it.
    pub(crate) original: &'a str,

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
                self.current_column = 0;
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
