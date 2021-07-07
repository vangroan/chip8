use super::{Parse, ParseError};
use crate::{
    token_stream::{TokenError, TokenStream},
    tokens::{Token, TokenKind},
};

#[derive(Debug)]
pub struct Comment {
    pub kind: CommentKind,
    pub token: Token,
    pub content: String,
}

#[derive(Debug)]
pub enum CommentKind {
    /// Comment line starting with `//`
    Line,
    /// Comment line starting with `///`
    Doc,
    /// Comment surrounded by `/*` and `*/`
    Block,
}

impl Parse for Comment {
    type Output = Self;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        input.reset_peek();

        let kind = match input.peek().map(|t| t.kind)? {
            TokenKind::Comment => CommentKind::Line,
            TokenKind::DocComment => CommentKind::Doc,
            token_kind => {
                return Err(ParseError::Token(TokenError::Unexpected {
                    encountered: token_kind,
                    msg: "expected a comment token '//', '///' or '/*'".to_owned(),
                }))
            }
        };

        let token = input.next_token().ok_or(ParseError::EOS)??;
        let content = input
            .fragment_span(&token.span)
            .map(|s| s.to_owned())
            .expect("identifier has no fragment");

        Ok(Self { kind, token, content })
    }
}
