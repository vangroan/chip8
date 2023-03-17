//! Assembler

use crate::error::{AsmError, Chip8Error, Chip8Result};

use super::{lexer::Lexer, tokens::TokenKind, Keyword, Token};

pub struct Assembler<'a> {
    lexer: Lexer<'a>,
    labels: Vec<(String, usize)>,
    bytecode: Vec<u8>,
}

impl<'a> Assembler<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer,
            labels: vec![],
            bytecode: vec![],
        }
    }

    pub fn parse(mut self) -> Chip8Result<Vec<u8>> {
        loop {
            let token = self.lexer.next_token();

            match token.kind {
                TokenKind::Newline => {
                    /* Empty line */
                    continue;
                }
                TokenKind::Dot => self.parse_label()?,
                TokenKind::Keyword(_) => self.parse_mnomenic(token)?,
                TokenKind::Unknown => {
                    let kind = token.kind.clone();
                    return Err(self.error(token, format!("unknown token {:?}", kind)));
                }
                TokenKind::EOF => break,
                _ => {
                    let message = format!("unsupported token {:?}", token.kind.clone());
                    return Err(self.error(token, message));
                }
            }
        }

        Ok(self.bytecode)
    }

    #[inline(never)]
    #[cold]
    fn error(&self, token: Token, message: impl ToString) -> Chip8Error {
        AsmError::new(self.lexer.source_code(), token, message).into()
    }

    fn next_offset(&self) -> usize {
        self.bytecode.len()
    }

    fn push_label(&mut self, name: &Token) {
        debug_assert_eq!(
            name.kind,
            TokenKind::Ident,
            "only identifiers may be used as label names"
        );

        self.labels.push((
            name.span.fragment(self.lexer.source_code()).to_owned(),
            self.next_offset(),
        ));
    }
}

impl<'a> Assembler<'a> {
    fn parse_label(&mut self) -> Chip8Result<()> {
        let name = self.lexer.next_token();
        if name.kind != TokenKind::Ident {
            return Err(self.error(name, "expected label name"));
        }

        let newline = self.lexer.next_token();
        if !matches!(newline.kind, TokenKind::Newline | TokenKind::EOF) {
            return Err(self.error(newline, "expected newline or end-of-file"));
        }

        self.push_label(&name);

        Ok(())
    }

    fn parse_mnomenic(&mut self, name: Token) -> Chip8Result<()> {
        if let TokenKind::Keyword(keyword) = name.kind {
            match keyword {
                Keyword::Load => self.parse_ld()?,
                _ => {
                    let fragment = name.span.fragment(self.lexer.source_code());
                    return Err(self.error(name, format!("unsupported opcode {:?}", fragment)));
                }
            }
        } else {
            let fragment = name.span.fragment(self.lexer.source_code());
            return Err(self.error(
                name,
                format!("expected keyword identifier, found {}", fragment),
            ));
        }

        Ok(())
    }

    /// LD
    fn parse_ld(&mut self) -> Chip8Result<()> {
        let dst = self.lexer.next_token();
        let _comma = self.lexer.next_token();
        let _src = self.lexer.next_token();
        Err(self.error(dst, "todo"))
    }
}
