use super::{stmts::DefStmt, Parse, ParseError};
use crate::{
    token_stream::{TokenError, TokenStream},
    tokens::TokenKind,
};

#[derive(Debug)]
pub struct Prog {
    pub stmts: Vec<DefStmt>,
}

impl Parse for Prog {
    type Output = Self;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        let mut stmts = vec![];
        let mut count = 0;

        loop {
            input.reset_peek();

            // if count > 100 {
            //     break;
            // }
            // count += 1;
            //
            // println!("next: {:?}", input.peek().map(|t| t.kind));
            // input.reset_peek();

            match input.peek() {
                Ok(token) => match token.kind {
                    TokenKind::Newline => {
                        // Empty line
                        input.consume(TokenKind::Newline)?;
                        continue;
                    }
                    TokenKind::EOS => break,
                    _ => stmts.push(DefStmt::parse(input)?),
                },
                Err(TokenError::EndOfSource) => break,
                Err(err) => todo!("parse error: {}", err),
            }
        }
        Ok(Prog { stmts })
    }
}
