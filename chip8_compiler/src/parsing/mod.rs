mod block;
mod expr;
mod stmts;
mod unit;
mod ident;

pub use ident::*;
pub use block::*;
pub use expr::*;
pub use stmts::*;
pub use unit::*;

use crate::lex::{TokenStream, TokenError};
use std::{error::Error, fmt};

pub trait Parse: Sized {
    type Output;
    type Err: Error;

    fn parse(input: &mut TokenStream) -> Result<Self::Output, Self::Err>;
}

#[derive(Debug)]
pub enum ParseError {
    Token(TokenError),
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "parse error")
    }
}

impl From<TokenError> for ParseError {
    #[inline]
    fn from(err: TokenError) -> Self {
        ParseError::Token(err)
    }
}
