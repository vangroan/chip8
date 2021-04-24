use std::{fmt, str::FromStr};

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Plus,       // `+`
    Minus,      // `-`
    Star,       // `*`
    Slash,      // `/`
    Eq,         // `=`
    EqEq,       // `==`
    Greater,    // `>`
    Lesser,     // `<`
    GreaterEq,  // `>=`
    LesserEq,   // `<=`
    Arrow,      // `->`
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

impl fmt::Display for TokenKind {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TokenKind as T;

        match self {
            T::Plus       => write!(f, "+"),
            T::Minus      => write!(f, "-"),
            T::Star       => write!(f, "*"),
            T::Eq         => write!(f, "="),
            T::EqEq       => write!(f, "=="),
            T::Slash      => write!(f, "/"),
            T::Greater    => write!(f, ">"),
            T::Lesser     => write!(f, "<"),
            T::GreaterEq  => write!(f, ">="),
            T::LesserEq   => write!(f, "<="),
            T::Arrow      => write!(f, "->"),
            T::Comment    => write!(f, "comment"),
            T::DocComment => write!(f, "doc-comment"),
            T::Colon      => write!(f, ":"),
            T::Semicolon  => write!(f, ";"),
            T::Number     => write!(f, "number"),
            T::Newline    => write!(f, "newline"),
            T::Ident      => write!(f, "ident"),
            T::Keyword(k) => fmt::Display::fmt(k, f),
            T::EOS        => write!(f, "end-of-source"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            "var" => Ok(K::Var),
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
