use super::{
    block::Block,
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
    pub keyword: Token,
    pub sig: FuncSig,
    pub body: FuncBody,
}

#[derive(Debug)]
pub struct FuncSig {
    pub ident: Ident,
    pub left_paren: Token,
    pub args: Delimited<ArgDef, Comma>,
    pub right_paren: Token,
}

#[derive(Debug)]
pub struct ArgDef {
    pub name: Ident,
    pub colon: Token,
    pub ty: Ident,
}

#[derive(Debug)]
pub struct FuncBody {
    pub left_brace: Token,
    pub block: Block,
    pub right_brace: Token,
}

impl Parse for FuncDef {
    type Output = Self;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        let keyword = input.consume(TokenKind::Keyword(KeywordKind::Func))?;
        let sig = FuncSig::parse(input)?;
        let body = FuncBody::parse(input)?;

        Ok(Self { keyword, sig, body })
    }
}

impl Parse for FuncSig {
    type Output = Self;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        let ident = Ident::parse(input)?;
        let left_paren = input.consume(TokenKind::LeftParen)?;
        let args = Delimited::<ArgDef, Comma>::parse(input)?;
        let right_paren = input.consume(TokenKind::RightParen)?;

        Ok(FuncSig {
            ident,
            left_paren,
            args,
            right_paren,
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

impl Parse for FuncBody {
    type Output = Self;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        let left_brace = input.consume(TokenKind::LeftBrace)?;
        let block = Block::parse(input)?;
        let right_brace = input.consume(TokenKind::RightBrace)?;

        Ok(FuncBody {
            left_brace,
            block,
            right_brace,
        })
    }
}
