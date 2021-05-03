//! Statement parsing.
use super::{expr::Expr, func::FuncDef, ident::Ident, Parse, ParseError};
use crate::{
    token_stream::{TokenError, TokenStream},
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
    /// Function definition
    Func(FuncDef),
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
    pub ty: DefTy,
    pub eq: Token,
    pub rhs: Expr,
    pub trail: Option<SyntaxTrivia>,
}

#[derive(Debug)]
pub struct VarDef {
    pub keyword: Token,
    pub name: String,
    pub ty: Option<DefTy>,
    pub eq: Option<Token>,
    pub rhs: Option<Expr>,
    pub trail: Option<SyntaxTrivia>,
}

/// Type specified in a definition statement.
///
/// In the following example the `: u8` is
/// part of the `DefTy` node.
#[derive(Debug)]
pub struct DefTy {
    pub colon: Token,
    pub ty: Ident,
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
                _ => {
                    return Err(ParseError::Generic {
                        msg: format!("unexpected keyword '{}'", keyword),
                    })
                }
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
        use TokenKind as T;

        let keyword = input.consume(T::Keyword(KeywordKind::Const))?;
        let ident = input.consume(T::Ident)?;
        let name = input
            .fragment_span(&ident.span)
            .map(|s| s.to_owned())
            .expect("identifier has no fragment");

        let ty = DefTy::parse(input)?;
        let eq = input.consume(T::Eq)?;
        let rhs = Expr::parse(input)?;

        Ok(Self {
            keyword,
            name,
            ty,
            eq,
            rhs,
            trail: None,
        })
    }
}

impl Parse for VarDef {
    type Output = Self;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        use TokenKind as T;

        let keyword = input.consume(T::Keyword(KeywordKind::Var))?;
        let ident = input.consume(T::Ident)?;
        let name = input
            .fragment_span(&ident.span)
            .map(|s| s.to_owned())
            .expect("identifier has no fragment");

        // Type is optional
        let ty = if input.peek_match(T::Colon) {
            Some(DefTy::parse(input)?)
        } else {
            None
        };
        // println!("ty: {:#?}", ty);

        // Previous peek was for optional token.
        // We need to peek the same position.
        input.reset_peek();

        // RHS is optional.
        let (eq, rhs) = if input.peek_match(T::Eq) {
            let eq = input.consume(T::Eq)?;
            let expr = Expr::parse(input)?;
            (Some(eq), Some(expr))
        } else {
            (None, None)
        };
        // println!("eq: {:#?}\nrhs: {:#?}", eq, rhs);

        // Either or both of the type or the RHS must be
        // present. If neither then invalid.
        if ty.is_none() && eq.is_none() && rhs.is_none() {
            Err(ParseError::Generic {
                msg: "variable definition must either specify a type or assign a value".to_owned(),
            })
        } else {
            Ok(VarDef {
                keyword,
                name,
                ty,
                eq,
                rhs,
                trail: None,
            })
        }
    }
}

impl Parse for DefTy {
    type Output = Self;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        Ok(DefTy {
            colon: input.consume(TokenKind::Colon)?,
            ty: Ident::parse(input)?,
        })
    }
}
