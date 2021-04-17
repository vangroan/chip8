use super::{Parse, ParseError};
use crate::token_stream::TokenStream;

#[derive(Debug)]
pub enum Expr {
    /// For development
    NoOp,
    Number,
}

impl Parse for Expr {
    type Output = Self;
    type Err = ParseError;

    fn parse(_input: &mut TokenStream) -> Result<Self, ParseError> {
        todo!()
    }
}
