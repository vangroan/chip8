//! Tokens

use std::ops;

#[derive(Debug)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[rustfmt::skip]
pub enum TokenKind {
    // Simple
    Comma,     // ,
    Dot,       // .
    Colon,     // :
    Semicolon, // ;
    /// Line-feed and optionally a carriage return
    Newline,

    // ------------------------------------------------------------------------
    // Complex
    Ident,
    /// Reserved identifiers
    Keyword(Keyword),
    /// String literal
    String,
    /// Number literal
    Number,

    // ------------------------------------------------------------------------
    // Special
    /// Unsupported token which should be treated as an error, probably
    Unknown,
    /// End-of-file
    EOF,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Span {
    pub index: u32,
    pub size: u32,
}

impl Span {
    pub fn new(index: u32, size: u32) -> Self {
        Self { index, size }
    }

    #[inline]
    pub fn fragment<'a>(&self, text: &'a str) -> &'a str {
        &text[(self.index as usize)..(self.index as usize + self.size as usize)]
    }

    /// Ending index of the span, exclusive.
    #[inline]
    pub fn end(&self) -> u32 {
        self.index + self.size
    }

    pub fn surrounding_line<'a>(&self, text: &'a str) -> (&'a str, Span) {
        const NEWLINE: char = '\n';
        const RETURN: char = '\r';

        let mut chars = text.char_indices().peekable();
        let mut start = 0;
        let mut end = text.len();

        while let Some((i, c)) = chars.next() {
            if i < self.index as usize {
                if c == NEWLINE {
                    // Span not found yet, move the starting cursor to the next line.

                    if chars.peek().map(|(_, c)| *c) == Some(RETURN) {
                        chars.next();
                    }

                    // Line starts at the character after the newline (\n) and carriage return (\r)
                    if let Some((i, _)) = chars.peek() {
                        start = *i;
                    }
                }
            } else if i >= self.end() as usize {
                if c == NEWLINE {
                    end = i + 1;

                    if chars.peek().map(|(_, c)| *c) == Some(RETURN) {
                        chars.next();
                        end += 1;
                    }

                    break;
                }
            }
        }

        let line_span = Span {
            index: start as u32,
            size: end as u32 - start as u32,
        };

        (&text[start..end], line_span)
    }
}

impl ops::Add for Span {
    type Output = Span;

    fn add(self, rhs: Self) -> Self::Output {
        let index = u32::min(self.index, rhs.index);
        let size = u32::max(self.end(), rhs.end()) - index;
        Span { index, size }
    }
}

/// Reserved keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[rustfmt::skip]
pub enum Keyword {
    // ------------------------------------------------------------------------
    // Opcodes
    Add,          // ADD
    And,          // AND
    Call,         // CALL
    Clear,        // CLS
    Draw,         // DRW
    Load,         // LD
    Jump,         // JP
    Or,           // OR
    ShiftLeft,    // SHR
    ShiftRight,   // SHR
    SkipEq,       // SE
    SkipEqNot,    // SNE
    SkipKey,      // SKP
    SkipKeyNot,   // SKNP
    Sub,          // SUB
    SubN,         // SUBN
    System,       // SYS
    Random,       // RND
    Return,       // RET
    Xor,          // XOR

    // ------------------------------------------------------------------------
    // Registers
    Char,      // F
    Decimal,   // BCD
    Delay,     // DT
    Index,     // I
    Key,       // K
    Sound,     // ST
}

impl Keyword {
    #[rustfmt::skip]
    pub fn parse(text: impl AsRef<str>) -> Option<Self> {
        match text.as_ref() {
            "add"  | "ADD"  => Some(Self::Add),
            "and"  | "AND"  => Some(Self::And),
            "call" | "CALL" => Some(Self::Call),
            "cls"  | "CLS"  => Some(Self::Clear),
            "drw"  | "DRW"  => Some(Self::Draw),
            "ld"   | "LD"   => Some(Self::Load),
            "jp"   | "JP"   => Some(Self::Jump),
            "or"   | "OR"   => Some(Self::Or),
            "shl"  | "SHL"  => Some(Self::ShiftLeft),
            "shr"  | "SHR"  => Some(Self::ShiftRight),
            "se"   | "SE"   => Some(Self::SkipEq),
            "sne"  | "SNE"  => Some(Self::SkipEqNot),
            "skp"  | "SKP"  => Some(Self::SkipKey),
            "sknp" | "SKNP" => Some(Self::SkipKeyNot),
            "sub"  | "SUB"  => Some(Self::Sub),
            "subn" | "SUBN" => Some(Self::SubN),
            "sys"  | "SYS"  => Some(Self::System),
            "rnd"  | "RAND" => Some(Self::Random),
            "ret"  | "RET"  => Some(Self::Return),
            "xor"  | "XOR"  => Some(Self::Xor),
            // ----------------------------------------------------------------
            "F"   => Some(Self::Char),
            "BCD" => Some(Self::Decimal),
            "DT"  => Some(Self::Delay),
            "K"   => Some(Self::Key),
            "ST"  => Some(Self::Sound),
            // ----------------------------------------------------------------
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_span_fragment() {
        const CODE: &str = "LD V0, 0xA4";

        let spans = &[
            Span::new(0, 2), // LD
            Span::new(3, 2), // V0
            Span::new(5, 1), // ,
            Span::new(7, 4), // 0xA4
        ];

        assert_eq!(spans[0].fragment(CODE), "LD");
        assert_eq!(spans[1].fragment(CODE), "V0");
        assert_eq!(spans[2].fragment(CODE), ",");
        assert_eq!(spans[3].fragment(CODE), "0xA4");
    }

    #[test]
    #[rustfmt::skip]
    fn test_span_surrounding_line() {
        const CODE: &str = "------------\n....here....\n------------";

        let span = Span::new(17, 4);
        assert_eq!(span.fragment(CODE), "here");

        let (line, line_span) = span.surrounding_line(CODE);
        assert_eq!(line, "....here....\n");
        assert_eq!(line_span, Span { index: 13, size: 13 });
    }

    #[test]
    #[rustfmt::skip]
    fn test_span_surrounding_line_cr() {
        const CODE: &str = "------------\n\r....here....\n\r------------";

        let span = Span::new(18, 4);
        assert_eq!(span.fragment(CODE), "here");

        let (line, line_span) = span.surrounding_line(CODE);
        assert_eq!(line, "....here....\n\r");
        assert_eq!(line_span, Span { index: 14, size: 14 });
    }

    #[test]
    #[rustfmt::skip]
    fn test_span_surrounding_full_text() {
        const CODE: &str = "....here....";

        let span = Span::new(4, 4);
        assert_eq!(span.fragment(CODE), "here");

        let (line, line_span) = span.surrounding_line(CODE);
        assert_eq!(line, "....here....");
        assert_eq!(line_span, Span { index: 0, size: 12 });
    }
}
