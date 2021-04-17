use super::{
    stmts::{ConstDef, Stmt, VarDef},
    Parse, ParseError,
};
use crate::{
    token_stream::{TokenError, TokenStream},
    tokens::{KeywordKind, TokenKind},
};

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

impl Parse for Block {
    type Output = Self;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        use KeywordKind as K;
        use TokenKind as T;

        let mut stmts = vec![];

        loop {
            input.reset_peek();

            match input.peek() {
                Ok(token) => match token.kind {
                    T::Newline => {
                        // Empty line
                        input.consume(T::Newline)?;
                        continue;
                    }
                    T::Keyword(keyword) => match keyword {
                        K::Const => stmts.push(ConstDef::parse(input).map(Stmt::Const)?),
                        K::Var => stmts.push(VarDef::parse(input).map(Stmt::Var)?),
                    },
                    T::EOS => break,
                    _ => {
                        // Expression statement
                        // stmts.push(Expr::parse(input).map());
                        input.next_token();
                    }
                },
                Err(TokenError::EndOfSource) => break,
                Err(err) => todo!("parse error: {}", err),
            }
        }

        Ok(Self { stmts })
    }
}
