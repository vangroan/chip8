//! Assembler

use crate::error::{AsmError, Chip8Error, Chip8Result};

use super::{lexer::Lexer, token_stream::TokenStream, tokens::TokenKind, Keyword, Token};

pub struct Assembler<'a> {
    stream: TokenStream<'a>,
    labels: Vec<(String, usize)>,
    bytecode: Vec<u8>,
}

impl<'a> Assembler<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            stream: TokenStream::new(lexer),
            labels: vec![],
            bytecode: vec![],
        }
    }

    pub fn parse(mut self) -> Chip8Result<Vec<u8>> {
        loop {
            match self.stream.peek_kind() {
                Some(token_kind) => {
                    match token_kind {
                        TokenKind::Newline => {
                            /* Skip empty line */
                            self.stream.consume(TokenKind::Newline)?;
                            continue;
                        }
                        TokenKind::Dot => self.parse_label()?,
                        TokenKind::Keyword(_) => self.parse_mnemonic()?,
                        TokenKind::Unknown => {
                            let token = self.stream.next_token().unwrap();
                            let message = format!("unknown token {:?}", token.kind);
                            return Err(self.error(token, message));
                        }
                        TokenKind::EOF => break,
                        _ => {
                            let token = self.stream.next_token().unwrap();
                            let message = format!("unsupported token {:?}", token.kind);
                            return Err(self.error(token, message));
                        }
                    }
                }
                None => break,
            }
        }

        Ok(self.bytecode)
    }

    #[inline(never)]
    #[cold]
    fn error(&self, token: Token, message: impl ToString) -> Chip8Error {
        AsmError::new(self.stream.source_code(), token, message).into()
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
            name.span.fragment(self.stream.source_code()).to_owned(),
            self.next_offset(),
        ));
    }
}

impl<'a> Assembler<'a> {
    const STATEMENT_END: &[TokenKind] = &[TokenKind::EOF, TokenKind::Newline];

    /// Consume an end-of-statement.
    fn consume_eos(&mut self) -> Chip8Result<()> {
        match self.stream.peek_kind() {
            Some(next_kind) => {
                for expected_kind in Self::STATEMENT_END.iter().cloned() {
                    if next_kind == expected_kind {
                        // Found
                        self.stream.next_token();
                        return Ok(());
                    } else {
                        continue;
                    }
                }
            }
            None => {
                // Lexer iterator is exhausted, meaning we're beyond EOF.
                return Ok(());
            }
        }

        let kind_names = Self::STATEMENT_END
            .iter()
            .map(|kind| format!("{:?}", kind))
            .collect::<Vec<_>>();
        panic!("expected one of: {}", kind_names.join(", "))
    }

    fn parse_label(&mut self) -> Chip8Result<()> {
        debug_assert!(matches!(self.stream.peek_kind(), Some(TokenKind::Dot)));

        let _dot = self.stream.consume(TokenKind::Dot)?;
        let name = self.stream.consume(TokenKind::Ident)?;
        if name.kind != TokenKind::Ident {
            return Err(self.error(name, "expected label name"));
        }

        self.consume_eos()?;

        self.push_label(&name);

        Ok(())
    }

    fn parse_mnemonic(&mut self) -> Chip8Result<()> {
        debug_assert!(matches!(
            self.stream.peek_kind(),
            Some(TokenKind::Keyword(_))
        ));

        let name = self.stream.next_token().ok_or_else(|| Chip8Error::EOF)?;

        if let TokenKind::Keyword(keyword) = name.kind {
            match keyword {
                Keyword::Load => self.parse_load(name)?,
                _ => {
                    let fragment = self.stream.span_fragment(&name.span);
                    return Err(self.error(name, format!("unsupported opcode {:?}", fragment)));
                }
            }
        }

        Ok(())
    }

    /// LD
    fn parse_load(&mut self, name: Token) -> Chip8Result<()> {
        let _dst = self.stream.next_token();
        let _comma = self.stream.next_token();
        let _src = self.stream.next_token();
        let _newline = self.stream.next_token();
        Err(self.error(name, "todo"))
    }
}
