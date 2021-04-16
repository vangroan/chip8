//! Compilation unit.
use super::{Parse, ParseError, block::Block};
use crate::lex::TokenStream;

#[derive(Debug)]
pub struct CompilationUnit {
    pub block: Block,
}

impl Parse for CompilationUnit {
    type Output = Self;
    type Err = ParseError;

    fn parse(_input: &TokenStream) -> Result<Self, ParseError> {
        todo!()
    }
}
