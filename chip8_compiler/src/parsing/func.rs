use super::{
    delim::{Comma, Delimited},
    ident::Ident,
    Parse, ParseError,
};
use crate::{
    token_stream::TokenStream,
    tokens::{KeywordKind, Token, TokenKind},
};

#[derive(Debug)]
pub struct FuncDef {
    keyword: Token,
    ident: Ident,
    left_parent: Token,
    args: Delimited<ArgDef, Comma>,
    right_parent: Token,
    left_brace: Token,
    right_brace: Token,
}

#[derive(Debug)]
pub struct ArgDef {
    pub name: Ident,
    pub colon: Token,
    pub ty: Ident,
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
        let args = Delimited::<ArgDef, Comma>::parse(input)?;
        let right_parent = input.consume(T::RightParen)?;

        let left_brace = input.consume(T::LeftBrace)?;
        // TODO: Function body block
        let right_brace = input.consume(T::RightBrace)?;

        Ok(Self {
            keyword,
            ident,
            left_parent,
            args,
            right_parent,
            left_brace,
            right_brace,
        })
    }
}

/// Parse one function argument definition.
///
/// Allowed to fail because the lookahead is here and not in the delimiter list.
impl Parse for ArgDef {
    type Output = Option<Self>;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Option<Self>, ParseError> {
        input.reset_peek();

        Ok(match input.peek().map(|t| t.kind)? {
            TokenKind::Ident => Some(ArgDef {
                name: Ident::parse(input)?,
                colon: input.consume(TokenKind::Colon)?,
                ty: Ident::parse(input)?,
            }),
            _ => None,
        })
    }
}
