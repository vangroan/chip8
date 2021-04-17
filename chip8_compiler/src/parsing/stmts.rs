//! Statement parsing.
use super::{expr::Expr, Parse, ParseError};
use crate::{
    lex::{TokenError, TokenStream},
    tokens::{KeywordKind, Token, TokenKind},
    trivia::SyntaxTrivia,
};

#[derive(Debug)]
pub enum Stmt {
    Comment,
    Const(ConstDef),
    /// Variable definition
    Var(VarDef),
    /// Expression Statements
    Expr(Expr),
}

/// Definition of constant value.
///
/// # Example
///
/// ```text
/// const FOO = 1;
/// ```
#[derive(Debug)]
pub struct ConstDef {
    pub keyword: Token,
    pub name: String,
    // TODO: Eq token
    pub rhs: Option<Expr>,
    pub trail: Option<SyntaxTrivia>,
}

#[derive(Debug)]
pub struct VarDef {
    pub keyword: Token,
    pub name: String,
    // TODO: Eq token
    pub rhs: Option<Expr>,
    pub trail: Option<SyntaxTrivia>,
}

impl Parse for Stmt {
    type Output = Self;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        use KeywordKind as K;
        use TokenKind as T;

        let result = match input.peek() {
            Ok(Token {
                kind: TokenKind::Keyword(keyword),
                ..
            }) => match keyword {
                K::Const => ConstDef::parse(input).map(Stmt::Const),
                K::Var => VarDef::parse(input).map(Stmt::Var),
            },
            Ok(Token { kind, .. }) => match kind {
                T::Comment | T::DocComment => {
                    // TODO: Comment token in returned statement node.
                    Ok(Stmt::Comment)
                }
                _ => panic!("unexpected token: {:?}", kind),
            },
            Err(TokenError::EndOfSource) => panic!("unexpected end-of-source"),
            _ => panic!(),
        };

        // Statement must be terminated with a semicolon.
        if result.is_ok() {
            input.match_token(T::Semicolon);
        }

        result
    }
}

impl Parse for ConstDef {
    type Output = Self;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        let keyword = input.consume(TokenKind::Keyword(KeywordKind::Const))?;
        let ident = input.consume(TokenKind::Ident)?;
        let name = input
            .fragment_span(&ident.span)
            .map(|s| s.to_owned())
            .expect("identifier has no fragment");

        // TODO: Parse optional type
        input.match_token(TokenKind::Colon);
        input.match_token(TokenKind::Ident);

        let rhs = input.match_token(TokenKind::Eq).then(|| Expr::NoOp);

        Ok(Self {
            keyword,
            name,
            rhs,
            trail: None,
        })
    }
}

impl Parse for VarDef {
    type Output = Self;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        let keyword = input.consume(TokenKind::Keyword(KeywordKind::Var))?;
        let ident = input.consume(TokenKind::Ident)?;
        let name = input
            .fragment_span(&ident.span)
            .map(|s| s.to_owned())
            .expect("identifier has no fragment");
        
        // TODO: Parse optional type
        input.match_token(TokenKind::Colon);
        input.match_token(TokenKind::Ident);
        
        let rhs = input.match_token(TokenKind::Eq).then(|| Expr::NoOp);

        Ok(Self {
            keyword,
            name,
            rhs,
            trail: None,
        })
    }
}
