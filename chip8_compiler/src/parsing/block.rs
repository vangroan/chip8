use super::{Parse, ParseError, stmts::Stmt};
use crate::lex::TokenStream;

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

impl Parse for Block {
    type Output = Self;
    type Err = ParseError;

    fn parse(_input: &TokenStream) -> Result<Self, ParseError> {
        todo!()
    }
}

