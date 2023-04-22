//! Assembler
mod assembler;
mod cursor;
mod lexer;
mod token_stream;
mod tokens;

use crate::error::Chip8Result;

pub fn assemble(source_code: impl AsRef<str>) -> Chip8Result<Vec<u8>> {
    let lexer = Lexer::new(source_code.as_ref());
    let asm = Assembler::new(lexer);
    asm.parse()
}

pub fn assemble_with(source_code: impl AsRef<str>, conf: AsmConf) -> Chip8Result<Vec<u8>> {
    let lexer = Lexer::new(source_code.as_ref());
    let asm = Assembler::with_conf(lexer, conf);
    asm.parse()
}

pub use self::{
    assembler::{AsmConf, Assembler},
    lexer::Lexer,
    tokens::{Keyword, Span, Token, TokenKind},
};
