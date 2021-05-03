mod block;
mod expr;
mod func;
mod ident;
mod literal;
mod stmts;
mod unit;
mod visitor;

pub use block::*;
pub use expr::*;
pub use func::*;
pub use ident::*;
pub use literal::*;
pub use stmts::*;
pub use unit::*;
pub use visitor::*;

use crate::token_stream::{TokenError, TokenStream};
use std::{error::Error, fmt, num::ParseIntError};

pub trait Parse: Sized {
    type Output;
    type Err: Error;

    fn parse(input: &mut TokenStream) -> Result<Self::Output, Self::Err>;
}

#[derive(Debug)]
pub enum ParseError {
    EOS,
    Generic { msg: String },
    Token(TokenError),
    Int(ParseIntError),
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParseError as E;
        write!(f, "parse error: ")?;
        match self {
            E::EOS => write!(f, "unexpected end-of-source"),
            E::Generic { msg } => write!(f, "{}", msg),
            E::Token(err) => fmt::Display::fmt(err, f),
            E::Int(err) => fmt::Display::fmt(err, f),
        }
    }
}

impl From<TokenError> for ParseError {
    #[inline]
    fn from(err: TokenError) -> Self {
        ParseError::Token(err)
    }
}

impl From<ParseIntError> for ParseError {
    #[inline]
    fn from(err: ParseIntError) -> Self {
        ParseError::Int(err)
    }
}
