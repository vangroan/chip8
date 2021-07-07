//! Compilation unit.
use super::{block::Block, Parse, ParseError};
use crate::token_stream::TokenStream;

#[derive(Debug)]
pub struct CompilationUnit {
    pub block: Block,
}

impl Parse for CompilationUnit {
    type Output = Self;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        Ok(Self {
            block: Block::parse(input)?,
        })
    }
}
