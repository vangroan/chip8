use super::{ident::Ident, Parse, ParseError};
use crate::{
    token_stream::TokenStream,
    tokens::{KeywordKind, Token, TokenKind},
};
use smol_str::SmolStr;

#[derive(Debug)]
pub struct FuncDef {
    keyword: Token,
    ident: Ident,
    left_parent: Token,
    right_parent: Token,
}

impl Parse for FuncDef {
    type Output = Self;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        use KeywordKind as K;
        use TokenKind as T;

        let keyword = input.consume(T::Keyword(K::Func))?;
        let ident = Ident::parse(input)?;
        let left_parent = input.consume(T::LeftParen)?;
        // TODO: Punctuated list
        let right_parent = input.consume(T::RightParen)?;

        Ok(Self {
            keyword,
            ident,
            left_parent,
            right_parent,
        })
    }
}
