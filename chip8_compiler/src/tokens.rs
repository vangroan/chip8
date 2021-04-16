use std::{fmt, str::FromStr};

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    Eq,         // `=`
    Slash,      // `/`
    Comment,    // `//`
    DocComment, // `///`
    Colon,      // `:`
    Semicolon,  // `;`

    /// Number Literal
    Number,

    /// Newline character, used for syntax trivia.
    Newline,

    Ident,

    /// Identifier  in the set of reserved words.
    Keyword(KeywordKind),

    /// End-of-source
    EOS,
}

#[derive(Debug, PartialEq, Eq)]
pub enum KeywordKind {
    Const,
    Var,
}

impl fmt::Display for KeywordKind {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use KeywordKind as K;
        match self {
            K::Const => write!(f, "const"),
            K::Var   => write!(f, "var"),
        }
    }
}

impl FromStr for KeywordKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use KeywordKind as K;
        match s {
            "const" => Ok(K::Const),
            _ => Err(()),
        }
    }
}

/// Chunk of source code, encoded as starting and ending positions.
#[derive(Debug, Clone)]
pub struct Span {
    /// Start position of bytes in source.
    pub start: usize,
    /// End position of bytes in source.
    pub end: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
}
