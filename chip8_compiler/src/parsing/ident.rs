// TODO: Identifier node
use super::{Parse, ParseError};
use crate::{
    token_stream::TokenStream,
    tokens::{Token, TokenKind},
};
use smol_str::SmolStr;
use std::fmt;

#[derive(Debug)]
pub struct Ident {
    pub token: Token,
    pub name: SmolStr,
}

impl fmt::Display for Ident {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Parse for Ident {
    type Output = Self;
    type Err = ParseError;

    #[inline]
    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        let token = input.consume(TokenKind::Ident)?;
        let name = input
            .fragment_span(&token.span)
            .map(|s| s.to_owned())
            .expect("identifier has no fragment")
            .into();
        Ok(Ident { token, name })
    }
}
