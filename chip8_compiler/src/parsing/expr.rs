use super::{Parse, ParseError};
use crate::lex::TokenStream;

#[derive(Debug)]
pub enum Expr {
    Number,
}

impl Parse for Expr {
    type Output = Self;
    type Err = ParseError;

    fn parse(_input: &TokenStream) -> Result<Self, ParseError> {
        todo!()
    }
}
