//! Result and errors.
use std::fmt::{self, Display, Formatter};

pub type Chip8Result<T> = std::result::Result<T, Chip8Error>;

#[derive(Debug)]
pub enum Chip8Error {
    /// VM error during interpreter loop.
    Runtime(&'static str),
    /// Attempt to load a bytecode program that can't fit in memory.
    LargeProgram,
    Fmt(fmt::Error),
}

impl Display for Chip8Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Runtime(msg) => write!(f, "runtime error: {}", msg),
            Self::LargeProgram => write!(f, "program too large for VM memory"),
            Self::Fmt(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Chip8Error {}

impl From<fmt::Error> for Chip8Error {
    fn from(err: fmt::Error) -> Self {
        Chip8Error::Fmt(err)
    }
}
