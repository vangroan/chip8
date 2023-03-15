mod block;
mod comment;
mod delim;
mod expr;
mod func;
mod ident;
mod literal;
mod prog;
mod stmts;
mod unit;
mod visitor;

pub use block::*;
pub use comment::*;
pub use delim::*;
pub use expr::*;
pub use func::*;
pub use ident::*;
pub use literal::*;
pub use prog::*;
pub use stmts::*;
pub use unit::*;
pub use visitor::*;

use crate::token_stream::{TokenError, TokenStream};
use std::{error::Error, fmt, fmt::Write as FmtWrite, num::ParseIntError};

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

impl ParseError {
    pub fn pretty_print<W: FmtWrite>(&self, writer: &mut W) {
        // TODO: Print file path, line and column
        // TODO: Print source line
        // TODO: Print arrow pointing at character
    }
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
