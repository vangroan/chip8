//! Assembler
use log::{debug, error, info, trace};

use crate::{
    bytecode::{opcodes::*, *},
    constants::*,
    error::{AsmError, Chip8Error, Chip8Result},
};

use super::{
    lexer::Lexer,
    token_stream::TokenStream,
    tokens::{Addr, NumFormat, Number, TokenKind},
    tokens::{Keyword, Span, Token},
};

/// Chip-8 assembler.
///
/// Because the semantics of the language are so simple,
/// this assembler is both parser frontend and codegen backend.
pub struct Assembler<'a> {
    /// Token stream of assembly code.
    stream: TokenStream<'a>,
    /// Symbol tabel of labels mapping to their target addresses.
    ///
    /// The address is stored as the proper nnn format used in Chip-8.
    labels: Vec<(String, u16)>,
    /// Record of attempts to access a label that hasn't been defined yet.
    ///
    /// Includes the token (and span) that attempted the access, as well
    /// as the index into the bytecode buffer where the corresponding
    /// instruction was emitted.
    ///
    /// This is then later used to patch the bytecode after all labels
    /// are be defined.
    ///
    /// See [`Assembler::fix_labels()`]
    defer: Vec<LabelAccess>,
    /// Result buffer of generated bytecode.
    bytecode: Vec<u8>,
    /// Collected errors.
    ///
    /// If an error is encountered in a statement, it is pushed onto this container.
    /// Parsing continues to collect further possible errors, but it
    /// has effectively failed the assembling run.
    errors: Vec<Chip8Error>,
}

/// Access to a label that hasn't been defined yet.
struct LabelAccess {
    /// The token where the label was accessed.
    token: Token,
    /// Index into the generated bytecode where the placeholder
    /// instruction was emitted.
    offset: usize,
}

impl<'a> Assembler<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            stream: TokenStream::new(lexer),
            labels: vec![],
            defer: vec![],
            bytecode: vec![],
            errors: vec![],
        }
    }

    pub fn parse(mut self) -> Chip8Result<Vec<u8>> {
        info!("assembling");
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
                        TokenKind::Number => self.parse_data_block()?,
                        TokenKind::Keyword(_) => self
                            .parse_mnemonic()
                            .or_else(|err| self.swallow_error(err))?,
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

        if self.has_errors() {
            return Err(Chip8Error::Multi(self.errors.drain(..).collect()));
        }

        let label_count = self.fix_labels()?;
        trace!("fixed {label_count} deferred labels");

        Ok(self.bytecode)
    }

    /// Build an assembly error.
    #[inline(never)]
    #[cold]
    fn error(&self, token: Token, message: impl ToString) -> Chip8Error {
        AsmError::new(self.stream.source_code(), token.span.clone(), message).into()
    }

    /// Build an end-of-file error that points to the end of the previous token.
    #[inline(never)]
    #[cold]
    fn eof_error(&self, expected: impl AsRef<str>) -> Chip8Error {
        // Format the error to point to the space after the previous token.
        let span = self
            .stream
            .previous_token()
            .map(|t| t.span.clone())
            .unwrap_or_else(|| Span::new(0, 1));
        log::warn!("previous token: {:?}", self.stream.previous_token());
        let expected = expected.as_ref();
        let message = format!("expected {expected}, but found end-of-file");
        AsmError::new(self.stream.source_code(), span, message).into()
    }

    /// Indicated whether any lines have enountered an error.
    fn has_errors(&self) -> bool {
        self.errors.len() > 0
    }

    /// Gobble up the served error and store it in our error belly. Yum, yum.
    fn swallow_error<T: Default>(&mut self, err: Chip8Error) -> Chip8Result<T> {
        use TokenKind as TK;

        // Collect error so it can be returned as a compound error later.
        self.errors.push(err);

        // Error encountered in the middle of a statement.
        // Consume rest of tokens until statement end.
        loop {
            // In the case where the error was on the newline,
            // we want to stop consuming immediately.
            // Otherwise we are consuming and discarding the next line.
            if self.stream.previous_token().map(|t| t.kind) == Some(TokenKind::Newline) {
                break;
            }

            match self.stream.peek_kind() {
                None | Some(TK::EOF) | Some(TK::Newline) => {
                    self.stream.next_token(); // consume newline
                    break;
                }
                _ => {
                    self.stream.next_token();
                }
            }
        }

        Ok(T::default())
    }

    /// Wraps a parslet and consumes the returned error.
    ///
    /// Returns `None` if an error occurred.
    fn try_line<T: Default>(&mut self, result: Chip8Result<T>) -> Chip8Result<T> {
        match result {
            Err(err) => self.swallow_error(err),
            Ok(value) => Ok(value),
        }
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

        // Target address that is being labeled.
        let address = (MEM_START + self.next_offset()) as u16;
        let fragment = name.span.fragment(self.stream.source_code()).to_owned();

        self.labels.push((fragment, address));
    }

    fn lookup_label(&self, name: &str) -> Option<u16> {
        self.labels
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, offset)| offset)
            .cloned()
    }

    /// Will store a deferred label access if the label cannot be found.
    ///
    /// IMPORTANT: The caller must emit a bytecode instruction immediately
    ///     after attempting to resolve a label.
    fn resolve_label(&mut self, label: Token) -> Option<u16> {
        debug_assert_eq!(
            label.kind,
            TokenKind::Label,
            "label must be resolved with a label token"
        );

        let name = self.stream.span_fragment(&label.span);
        let maybe_nnn = self.lookup_label(name);

        // If the label is accessed before it's defined, then
        // the caller is expected to emit a placeholder instruction.
        //
        // The access to the label is stored with enough bookkeeping
        // to replace the instruction later.
        if maybe_nnn.is_none() {
            let next_offset = self.next_offset();
            self.defer.push(LabelAccess {
                token: label,
                offset: next_offset,
            });
        }

        maybe_nnn
    }

    fn emit2(&mut self, instr: [u8; 2]) {
        trace!("emit2: {:02X} {:02X}", instr[0], instr[1]);
        self.bytecode.push(instr[0]);
        self.bytecode.push(instr[1]);
    }

    fn emit(&mut self, instr: u8) {
        trace!("emit: {instr:02X}");
        self.bytecode.push(instr);
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

/// Label definition pass.
impl<'a> Assembler<'a> {
    /// Pass over the generated bytecode to replace placeholder
    /// instructions with the actual address of the defined label.
    fn fix_labels(&mut self) -> Chip8Result<usize> {
        info!("patching label addresses");

        let mut count = 0;

        // Dump labels
        debug!("labels:");
        for (name, nnn) in &self.labels {
            debug!("    .{name}: 0x{nnn:X}");
        }

        // Take accesses waiting for label to be defined.
        let deferred_access: Vec<_> = self.defer.drain(..).collect();

        for access in deferred_access {
            debug_assert_eq!(access.token.kind, TokenKind::Label);

            let name = self.stream.span_fragment(&access.token.span);
            let nnn = self.lookup_label(name).ok_or_else(|| {
                let message = format!("label '{name}' is undefined");
                self.error(access.token, message)
            })?;

            self.patch_nnn(access.offset, nnn)?;

            count += 1;
        }

        Ok(count)
    }

    /// Replace the placeholder nnn in the instruction at the given index.
    fn patch_nnn(&mut self, index: usize, nnn: u16) -> Chip8Result<()> {
        trace!("patch_nnn: replacing instruction at {index} with 0x{nnn:X}");
        assert!(
            index + 1 < self.bytecode.len(),
            "out-of-range attempt to patch bytecode"
        );

        let a = self.bytecode[index];
        let _ = self.bytecode[index + 1]; // 00

        self.bytecode[index] = (a & 0b11110000) | (nnn >> 8) as u8;
        self.bytecode[index + 1] = (nnn & 0b11111111) as u8;

        Ok(())
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

        let token = self
            .stream
            .next_token()
            .unwrap_or_else(|| unreachable!("end-of-file already checked"));
        let message = format!(
            "expected end-of-file or newline, but found {:?}",
            token.kind
        );
        Err(self.error(token, message))
    }

    /// Parse the `nnn` argument of a mnemonic, which
    /// holds an absolute memory address as either a
    /// number literal or a label.
    fn parse_nnn(&mut self) -> Chip8Result<Addr> {
        let token = self
            .stream
            .next_token()
            .ok_or_else(|| self.eof_error("an address as either a number literal or label"))?;

        match token.kind {
            TokenKind::Number => {
                let number = self.parse_number(token)?;

                Ok(Addr::Num(number))
            }
            TokenKind::Dot => {
                let ident = self.stream.consume(TokenKind::Ident)?;

                // Transform the identifier into a label for ease of use.
                // Technically the grammar is now no longer context-free.
                let label = Token {
                    kind: TokenKind::Label,
                    // FIXME: merging spans breaks label lookup later.
                    // span: nnn.span + ident.span,
                    span: ident.span,
                };

                Ok(Addr::Label(label))
            }
            _ => {
                let kind = token.kind;
                Err(self.error(token, format!("expected an address as either a number literal or label, but found {kind:?}")))
            }
        }
    }

    fn parse_arg2(&mut self) -> Chip8Result<[Token; 2]> {
        let dst = self.stream.next_token().ok_or_else(|| Chip8Error::EOF)?;
        let _comma = self.stream.consume(TokenKind::Comma)?;
        let src = self.stream.next_token().ok_or_else(|| Chip8Error::EOF)?;

        match src.kind {
            TokenKind::Number | TokenKind::Keyword(_) => Ok([dst, src]),
            TokenKind::Dot => {
                let ident = self.stream.consume(TokenKind::Ident)?;

                // Transform the identifier into a label for ease of use.
                // Technically the grammar is now no longer context-free.
                let nnn = Token {
                    kind: TokenKind::Label,
                    // FIXME: merging spans breaks label lookup later.
                    // span: nnn.span + ident.span,
                    span: ident.span,
                };

                Ok([dst, nnn])
            }
            _ => {
                let kind = src.kind;
                let message = format!("expected number literal or label, but found {kind:?}");
                Err(self.error(src, message))
            }
        }
    }

    fn parse_xnn(&mut self) -> Chip8Result<(u8, Number)> {
        let vx = self
            .stream
            .next_token()
            .ok_or_else(|| Chip8Error::EOF)
            .and_then(|t| self.parse_vregister(t))?;
        let _comma = self.stream.consume(TokenKind::Comma)?;
        let nn = self
            .stream
            .consume(TokenKind::Number)
            .and_then(|t| self.parse_number(t))?;

        Ok((vx, nn))
    }

    fn parse_xyn(&mut self) -> Chip8Result<(u8, u8, Number)> {
        let vx = self
            .stream
            .next_token()
            .ok_or_else(|| Chip8Error::EOF)
            .and_then(|t| self.parse_vregister(t))?;
        let _comma = self.stream.consume(TokenKind::Comma)?;
        let vy = self
            .stream
            .next_token()
            .ok_or_else(|| Chip8Error::EOF)
            .and_then(|t| self.parse_vregister(t))?;
        let _comma = self.stream.consume(TokenKind::Comma)?;
        let n = self
            .stream
            .consume(TokenKind::Number)
            .and_then(|t| self.parse_number(t))?;

        Ok((vx, vy, n))
    }

    fn parse_vregister(&self, token: Token) -> Chip8Result<u8> {
        use TokenKind as TK;

        match token.kind {
            TK::Keyword(keyword) => {
                // don't format me
                match keyword.as_vregister() {
                    Some(vregister) => Ok(vregister),
                    None => {
                        let message = format!(
                            "expected one of the V0-VF registers, but found {:?}",
                            token.kind
                        );
                        Err(self.error(token, message))
                    }
                }
            }
            TK::EOF => Err(self.eof_error("one of the V0-VF registers")),
            _ => {
                let message = format!(
                    "expected one of the V0-VF registers, but found {:?}",
                    token.kind
                );
                Err(self.error(token, message))
            }
        }
    }

    fn parse_number(&self, token: Token) -> Chip8Result<Number> {
        use NumFormat as NF;

        trace!("parse_number");
        debug_assert_eq!(token.kind, TokenKind::Number);

        let fragment = self.stream.span_fragment(&token.span);
        trace!("fragment {fragment}");

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

        Ok(Number {
            token,
            value,
            format,
        })
    }

    fn parse_label(&mut self) -> Chip8Result<()> {
        trace!("parse_label");
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

    /// Emit raw data into bytecode.
    fn parse_data_block(&mut self) -> Chip8Result<()> {
        trace!("parse data block");
        debug_assert!(self.bytecode.len() % 2 == 0);

        let mut count = 0;
        let mut last_token: Option<Token> = None;

        loop {
            match self.stream.peek_kind() {
                Some(TokenKind::Number) => {
                    let token = self.stream.consume(TokenKind::Number)?;
                    let nn = self.parse_number(token)?;
                    if nn.value > u8::MAX as u16 {
                        panic!("only 8-bit literals are currently supported");
                    }
                    self.emit(nn.value as u8);
                    last_token = Some(nn.token);
                    count += 1;
                }
                _ => break,
            }

            // Discard optional newline so we can continue consuming data
            // split accross multiple lines.
            let _newline = self.stream.consume(TokenKind::Newline);
        }

        trace!("data count: {count}");

        // Stride of bytecode must be 2 for program counter to increment correctly.
        if let Some(token) = last_token {
            if count % 2 != 0 {
                // Place the error message at the last data literal.
                // FIXME: Error format that will show preceding lines.
                return Err(self.error(token, "data must be added in 2 byte pairs"));
            }
        }

        Ok(())
    }

    fn parse_mnemonic(&mut self) -> Chip8Result<()> {
        use Keyword as KW;

        trace!("parse mnemonic");
        debug_assert!(matches!(
            self.stream.peek_kind(),
            Some(TokenKind::Keyword(_))
        ));

        let name = self.stream.next_token().ok_or_else(|| Chip8Error::EOF)?;

        if let TokenKind::Keyword(keyword) = name.kind {
            match keyword {
                KW::Add => self.parse_add(name)?,
                KW::Call => self.parse_call(name)?,
                KW::Clear => self.parse_clear_screen(name)?,
                KW::Draw => self.parse_draw(name)?,
                KW::Jump => self.parse_jump(name)?,
                KW::Load => self.parse_load(name)?,
                KW::Random => self.parse_random(name)?,
                KW::Return => self.parse_return(name)?,
                KW::SkipEq => self.parse_skip_eq(name)?,
                KW::SkipEqNot => self.parse_skip_neq(name)?,
                _ => {
                    let fragment = self.stream.span_fragment(&name.span);
                    return Err(self.error(name, format!("unsupported opcode {:?}", fragment)));
                }
            }
        }

        Ok(())
    }

    fn parse_clear_screen(&mut self, name: Token) -> Chip8Result<()> {
        trace!("parse_clear_screen");
        debug_assert_eq!(name.kind, TokenKind::Keyword(Keyword::Clear));
        self.emit2(encode_bare(CLS));
        Ok(())
    }

    fn parse_return(&mut self, name: Token) -> Chip8Result<()> {
        trace!("parse_return");
        debug_assert_eq!(name.kind, TokenKind::Keyword(Keyword::Return));
        self.emit2(encode_bare(RET));
        Ok(())
    }

    /// Parse Jump
    ///
    /// 1nnn (JP addr)
    /// Bnnn (JP V0, addr)
    fn parse_jump(&mut self, name: Token) -> Chip8Result<()> {
        trace!("parse_jump");
        debug_assert_eq!(name.kind, TokenKind::Keyword(Keyword::Jump));

        // Jump can optionally take the V0 register as an offset.
        let opcode: u8 = {
            match self
                .stream
                .peek_kind()
                .ok_or_else(|| self.eof_error("register, number literal or label"))?
            {
                TokenKind::Keyword(keyword) if keyword.is_vregister() => {
                    match keyword {
                        Keyword::V0 => {
                            let _v0 = self.stream.consume(TokenKind::Keyword(Keyword::V0))?;
                            let _comma = self.stream.consume(TokenKind::Comma)?;
                            JP_V0_ADDR
                        }
                        other => {
                            // V1-VF
                            let token = self.stream.next_token().unwrap();
                            let message = format!(
                                "only register V0 is supported as a jump offset, not {other:?}"
                            );
                            return Err(self.error(token, message));
                        }
                    }
                }
                _ => JP_ADDR,
            }
        };

        let nnn = self.parse_nnn()?;

        match nnn {
            Addr::Num(number) => {
                if number.value > 0xFFF {
                    return Err(self.error(
                        number.token.clone(),
                        "argument for jump address must be 12-bits",
                    ));
                }
                self.emit2(encode_nnn(opcode, number.value));
            }
            Addr::Label(label) => {
                // NOTE: If label is not defined yet,address 0x000 is inserted as a placeholder.
                //       Error handling is in the fix_labels pass.
                let number = self.resolve_label(label).unwrap_or_default() & 0xFFF;
                self.emit2(encode_nnn(opcode, number));
            }
        }
        Ok(())
    }

    /// Parse Call
    ///
    /// 2NNN (CALL addr)
    fn parse_call(&mut self, name: Token) -> Chip8Result<()> {
        trace!("parse_call");
        debug_assert_eq!(name.kind, TokenKind::Keyword(Keyword::Call));

        let nnn = self.parse_nnn()?;

        match nnn {
            Addr::Num(number) => {
                if number.value > 0xFFF {
                    return Err(self.error(
                        number.token.clone(),
                        "argument for call address must be 12-bits",
                    ));
                }
                self.emit2(encode_nnn(CALL_ADDR, number.value));
            }
            Addr::Label(label) => {
                // NOTE: If label is not defined yet,address 0x000 is inserted as a placeholder.
                //       Error handling is in the fix_labels pass.
                let number = self.resolve_label(label).unwrap_or_default() & 0xFFF;
                self.emit2(encode_nnn(CALL_ADDR, number));
            }
        }

        Ok(())
    }

    /// 3XNN (SE Vx, byte)
    /// 5xy0 (SE Vx, Vy)
    fn parse_skip_eq(&mut self, _name: Token) -> Chip8Result<()> {
        use Keyword as KW;
        use TokenKind as TK;

        trace!("parse_skip_eq");

        let [lhs, rhs] = self.parse_arg2()?;
        let signature = [lhs.kind, rhs.kind];
        match signature {
            [TK::Keyword(kw), TK::Number] if kw.is_vregister() => {
                let vx = self.parse_vregister(lhs)?;
                let nn = self.parse_number(rhs)?;
                self.emit2(encode_xnn(SE_VX_NN, vx, nn.as_u8()));
            }
            [TK::Keyword(kw1), TK::Keyword(kw2)] if kw1.is_vregister() && kw2.is_vregister() => {
                let vx = self.parse_vregister(lhs)?;
                let vy = self.parse_vregister(rhs)?;
                self.emit2(encode_xyn(SE_VX_VY, vx, vy, 0));
            }
            [TK::Keyword(kw), _] if kw.is_vregister() => {
                let kind = rhs.kind;
                return Err(self.error(
                    rhs,
                    format!("expected register or number literal, but found {kind:?}"),
                ));
            }
            _ => {
                let kind = lhs.kind;
                return Err(self.error(rhs, format!("expected register, but found {kind:?}")));
            }
        }
        // let (vx, nn) = self.parse_xnn()?;
        self.consume_eos()?;
        // self.emit2(encode_xnn(SE_VX_NN, vx, nn.as_u8()));
        Ok(())
    }

    /// 4xNN (SNE Vx, byte)
    fn parse_skip_neq(&mut self, _name: Token) -> Chip8Result<()> {
        trace!("parse_skip_neq");

        let (vx, nn) = self.parse_xnn()?;
        self.consume_eos()?;
        self.emit2(encode_xnn(SNE_VX_NN, vx, nn.as_u8()));
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
    fn parse_load(&mut self, name: Token) -> Chip8Result<()> {
        use Keyword as KW;
        use TokenKind as TK;

        trace!("parse_load");
        debug_assert_eq!(name.kind, TK::Keyword(KW::Load));

        // let dst = self.stream.next_token().ok_or_else(|| Chip8Error::EOF)?;
        let [dst, src] = self.parse_arg2()?;

        let signature = [dst.kind, src.kind];

        match signature {
            // 6XNN (LD Vx, byte)
            //
            // Load byte literal into Vx register
            [TK::Keyword(kw), TK::Number] if kw.is_vregister() => {
                let vx = kw.as_vregister().unwrap_or_else(|| unreachable!());
                let nn = self.parse_number(src)?;
                self.emit2(encode_xnn(LD_VX_NN, vx, nn.as_u8()))
            }
            // ANNN (LD I, addr)
            //
            // Load memory address into index register.
            [TK::Keyword(KW::Index), TK::Number] => {
                let nnn = self.parse_number(src)?;
                self.emit2(encode_nnn(LD_I_NNN, nnn.value));
            }
            // ANNN (LD I, label)
            //
            // Load memory address into index register.
            [TK::Keyword(KW::Index), TK::Label] => {
                // NOTE: If label is not defined yet, we default to 0x000
                let nnn = (self.resolve_label(src).unwrap_or_default() & 0xFFF) as u16;
                self.emit2(encode_nnn(LD_I_NNN, nnn));
            }
            // Fx07 (LD Vx,  DT)
            //
            // Load delay timer into Vx register
            [TK::Keyword(kw), TK::Keyword(KW::Delay)] if kw.is_vregister() => {
                return Err(self.error(src, "not implemented yet"));
            }
            // Fx55 (LD [I], Vx)
            //
            // Load registers into memory block.
            [TK::Keyword(KW::Index), TK::Keyword(kw)] if kw.is_vregister() => {
                return Err(self.error(dst, "not implemented yet"));
            }
            [TK::Keyword(kw), _] if kw.is_vregister() => {
                let kind = src.kind;
                let message = format!("expected byte literal, but found {kind:?}");
                return Err(self.error(src, message));
            }
            [TK::Keyword(_), _] => {
                let kind = src.kind;
                let message = format!("expected address, label or register, but found {kind:?}");
                return Err(self.error(src, message));
            }
            _ => {
                let message = format!(
                    "unsupported arguments, found {:?}, {:?}",
                    dst.kind, src.kind
                );
                return Err(self.error(dst, message));
            }
        }

        self.consume_eos()?;

        Ok(())
    }

    fn parse_add(&mut self, _name: Token) -> Chip8Result<()> {
        let (vx, nn) = self.parse_xnn()?;
        self.consume_eos()?;
        self.emit2(encode_xnn(ADD_VX_NN, vx, nn.as_u8()));
        Ok(())
    }

    fn parse_random(&mut self, _name: Token) -> Chip8Result<()> {
        let (vx, nn) = self.parse_xnn()?;
        self.consume_eos()?;
        self.emit2(encode_xnn(RND_X_NN, vx, nn.as_u8()));
        Ok(())
    }

    fn parse_draw(&mut self, _name: Token) -> Chip8Result<()> {
        let (vx, vy, n) = self.parse_xyn()?;
        if n.value > 0xF {
            return Err(self.error(n.token, "argument must be 15 or less (<= 0xF)"));
        }
        self.consume_eos()?;
        self.emit2(encode_xyn(DRW_X_Y_N, vx, vy, n.as_u8()));
        Ok(())
    }
}

fn slice_number(fragment: &str) -> &str {
    let rest = &fragment[2..];
    trace!("slice_number: fragment {fragment} rest {rest}");
    if rest == "" {
        "0"
    } else {
        rest
    }
}
