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

    fn resolve_label(&mut self, label: Token) -> Chip8Result<usize> {
        let query = self.stream.span_fragment(&label.span);
        self.labels
            .iter()
            .find(|(name, _)| name == query)
            .map(|(_, offset)| offset)
            .cloned()
            .ok_or_else(|| self.error(label, "label is undefined"))
    }

    fn emit(&mut self, instr: [u8; 2]) {
        println!("push: {:02X} {:02X}", instr[0], instr[1]);
        self.bytecode.push(instr[0]);
        self.bytecode.push(instr[1]);
    }

    fn dump_bytecode(&self) {
        // Instructions are always 2 bytes.
        assert!(self.bytecode.len() % 2 == 0);

        for (i, instr) in self.bytecode.chunks(2).enumerate() {
            let offset = i * 2;
            let a = instr[0];
            let b = instr[1];
            println!("0x{offset:04X} {a:02X}{b:02X}");
        }
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

    fn parse_args(&mut self) -> Chip8Result<[Token; 2]> {
        let dst = self.stream.next_token().ok_or_else(|| Chip8Error::EOF)?;
        let _comma = self.stream.consume(TokenKind::Comma)?;
        let mut src = self.stream.next_token().ok_or_else(|| Chip8Error::EOF)?;

        // Labels start with a dot
        if src.kind == TokenKind::Dot {
            src = self.stream.consume(TokenKind::Ident)?;

            // Transform the identifier into a label for ease of use.
            // Technically the grammar is now no longer context-free.
            src.kind = TokenKind::Label;
        }

        Ok([dst, src])
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
    /// - 6XNN (LD Vx,  byte)
    /// - ANNN (LD I,   addr)
    /// - Fx07 (LD Vx,  DT)
    /// - Fx0A (LD Vx,  K)
    /// - Fx15 (LD DT,  Vx)
    /// - Fx18 (LD ST,  Vx)
    /// - Fx29 (LD F,   Vx)
    /// - Fx33 (LD B,   Vx)
    /// - Fx55 (LD [I], Vx)
    /// - Fx65 (LD Vx,  [I])
    fn parse_load(&mut self, _name: Token) -> Chip8Result<()> {
        use Keyword as KW;
        use TokenKind as TK;

        // let dst = self.stream.next_token().ok_or_else(|| Chip8Error::EOF)?;
        let [dst, src] = self.parse_args()?;

        let signature = [dst.kind, src.kind];

        match signature {
            // (LD Vx, ____)
            //
            // Load byte literal into Vx register
            [TK::Keyword(kw), TK::Number] if is_vregister(kw) => {
                let vx = kw.as_vregister().unwrap_or_else(|| unreachable!());
                let nn = self.parse_number(src)?;
                self.emit(encode_xnn(LD_VX_BYTE, vx, nn.as_u8()))
            }
            // ANNN (LD I, addr)
            //
            // Load memory address into index register.
            [TK::Keyword(KW::Index), TK::Number] => {
                let nnn = self.parse_number(src)?;
                self.emit(encode_nnn(LD_NNN_BYTE, nnn.value));
            }
            // ANNN (LD I, label)
            //
            // Load memory address into index register.
            [TK::Keyword(KW::Index), TK::Label] => {
                let nnn = (self.resolve_label(src)? & 0xFFF) as u16;
                self.emit(encode_nnn(LD_NNN_BYTE, nnn));
            }
            // Fx55 (LD [I], Vx)
            //
            // Load registers into memory block.
            [TK::Keyword(KW::Index), TK::Keyword(kw)] if is_vregister(kw) => {}
            _ => {
                let message = format!(
                    "unsupported arguments, found {:?}, {:?}",
                    dst.kind, src.kind
                );
                return Err(self.error(dst, message));
            }
        }

        let _newline = self.stream.consume(TokenKind::Newline);

        // println!("{:?}", self.bytecode);
        self.dump_bytecode();

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
