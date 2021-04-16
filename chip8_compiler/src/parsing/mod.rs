mod stmts;
mod unit;
mod block;
mod expr;

pub use expr::*;
pub use stmts::*;
pub use unit::*;
pub use block::*;

use crate::lex::TokenStream;
use std::{error::Error, fmt};

pub trait Parse: Sized {
    type Output;
    type Err: Error;

    fn parse(input: &TokenStream) -> Result<Self::Output, Self::Err>;
}

#[derive(Debug)]
pub struct ParseError {}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "parse error")
    }
}
