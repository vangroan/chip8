//! Delimited list.
use super::{Parse, ParseError};
use crate::{
    token_stream::TokenStream,
    tokens::{Token, TokenKind},
};

#[derive(Debug)]
pub struct Delimited<T, D> {
    pub items: Vec<DelimitedItem<T, D>>,
}

#[derive(Debug)]
pub struct DelimitedItem<T, D> {
    pub item: T,
    pub delim: Option<D>,
}

#[derive(Debug)]
pub struct Comma {
    pub token: Token,
}

impl<T, D> Parse for Delimited<T, D>
where
    T: Parse<Output = Option<T>, Err = ParseError>,
    D: Parse<Output = Option<D>, Err = ParseError>,
{
    type Output = Self;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        let mut items = vec![];

        while let Some(item) = T::parse(input)? {
            items.push(DelimitedItem {
                item,
                delim: D::parse(input)?,
            });
        }

        Ok(Delimited { items })
    }
}

/// Parse a comma token into an AST node.
///
/// Allowed to fail because the lookahead is here and not in the delimiter list.
impl Parse for Comma {
    type Output = Option<Self>;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Option<Self>, ParseError> {
        input.reset_peek();
        Ok(match input.peek().map(|t| t.kind)? {
            TokenKind::Comma => Some(Comma {
                token: input.consume(TokenKind::Comma)?,
            }),
            _ => None,
        })
    }
}
