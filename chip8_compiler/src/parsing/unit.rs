//! Compilation unit.
use super::{block::Block, Parse, ParseError};
use crate::lex::TokenStream;

#[derive(Debug)]
pub struct CompilationUnit {
    pub block: Block,
}

impl Parse for CompilationUnit {
    type Output = Self;
    type Err = ParseError;

    fn parse(input: &TokenStream) -> Result<Self, ParseError> {
        // while let Some(token) = input.peek() {

        // }
        todo!()
    }
}
