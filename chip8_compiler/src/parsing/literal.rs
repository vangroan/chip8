//! Literal values.
use super::{Parse, ParseError};
use crate::{
    token_stream::{TokenError, TokenStream},
    tokens::{Token, TokenKind},
};

#[derive(Debug)]
pub struct Literal {
    pub token: Token,
    pub value: LitValue,
}

#[derive(Debug)]
pub enum LitValue {
    U8(u8),
}

impl Parse for Literal {
    type Output = Self;
    type Err = ParseError;

    #[inline]
    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        use TokenKind as T;

        input.reset_peek();
        match input.peek().map(|t| t.kind)? {
            T::Number => {
                // Only one number type.
                let token = input.consume(T::Number)?;
                let value = input
                    .fragment_span(&token.span)
                    .map(|s| s.to_owned())
                    .expect("identifier has no fragment")
                    .parse::<u8>()
                    .map(LitValue::U8)?;
                Ok(Literal { token, value })
            }
            token_kind => Err(ParseError::Token(TokenError::Unexpected {
                encountered: token_kind,
                msg: "literal expected".to_owned(),
            })),
        }
    }
}
