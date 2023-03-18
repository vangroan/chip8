//! Assembler

use crate::{
    bytecode::{opcodes::*, *},
    error::{AsmError, Chip8Error, Chip8Result},
};

use super::{
    lexer::Lexer,
    token_stream::TokenStream,
    tokens::{NumFormat, Number, TokenKind},
    Keyword, Token,
};

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
                            let message = format!("expected opcode, found {:?}", token.kind);
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

    fn emit(&mut self, instr: [u8; 2]) {
        println!("push: {:02X} {:02X}", instr[0], instr[1]);
        self.bytecode.push(instr[0]);
        self.bytecode.push(instr[1]);
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

    fn parse_number(&self, token: Token) -> Chip8Result<Number> {
        use NumFormat as NF;

        let fragment = self.stream.span_fragment(&token.span);
        println!("fragment {fragment}");

        // All digits are ASCII, so we can cast the UTF-8 string to bytes
        // and treat every byte as a character.
        let bytes = fragment.as_bytes();

        let (format, parse_result) = if bytes[0] == b'0' {
            match bytes.get(1) {
                Some(b'b') => (NF::Bin, u16::from_str_radix(slice_number(fragment), 2)),
                Some(b'x') => (NF::Hex, u16::from_str_radix(slice_number(fragment), 16)),
                _ => (NF::Dec, u16::from_str_radix(fragment, 10)),
            }
        } else {
            (NF::Dec, u16::from_str_radix(fragment, 10))
        };

        let value = parse_result.map_err(|err| Chip8Error::NumberParse(err))?;

        Ok(Number { value, format })
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

    /// Load
    ///
    /// - 6XNN (LD Vx, byte)
    /// - ANNN (LD I, addr)
    /// - Fx07 (LD Vx, DT)
    /// - Fx0A (LD Vx, K)
    /// - Fx15 (LD DT, Vx)
    /// - Fx18 (LD ST, Vx)
    /// - Fx29 (LD F, Vx)
    /// - Fx33 (LD B, Vx)
    /// - Fx55 (LD [I], Vx)
    /// - Fx65 (LD Vx, [I])
    fn parse_load(&mut self, name: Token) -> Chip8Result<()> {
        let dst = self.stream.next_token().ok_or_else(|| Chip8Error::EOF)?;

        match dst.kind {
            TokenKind::Keyword(kw) => match kw {
                // 6XNN (LD Vx, byte)
                //
                // Load byte literal into Vx register
                kw if is_vregister(kw) => {
                    let vx = kw.as_vregister().unwrap_or_else(|| unreachable!());
                    self.stream.consume(TokenKind::Comma)?;
                    let literal = self.stream.next_token().ok_or_else(|| Chip8Error::EOF)?;
                    let nn = self.parse_number(literal)?;
                    self.emit(encode_xnn(LD_VX_BYTE, vx, nn.as_u8()))
                }
                _ => return Err(self.error(name, "todo")),
            },
            _ => return Err(self.error(name, "todo")),
        }

        let _newline = self.stream.consume(TokenKind::Newline);

        println!("{:?}", self.bytecode);

        // Err(self.error(name, "todo"))
        Ok(())
    }
}

fn is_vregister(keyword: Keyword) -> bool {
    use Keyword as K;
    matches!(
        keyword,
        K::V0
            | K::V1
            | K::V2
            | K::V3
            | K::V4
            | K::V5
            | K::V6
            | K::V7
            | K::V8
            | K::V9
            | K::VA
            | K::VB
            | K::VC
            | K::VD
            | K::VE
            | K::VF
    )
}

fn slice_number(fragment: &str) -> &str {
    let rest = &fragment[2..];
    println!("fragment {fragment} rest {rest}");
    if rest == "" {
        "0"
    } else {
        rest
    }
}
