//! Result and errors.
use std::fmt::{self, Display, Formatter};

pub type Result<T> = std::result::Result<T, Chip8Error>;

#[derive(Debug)]
pub enum Chip8Error {
    /// Attempt to load a bytecode program that can't fit in memory.
    LargeProgram,
}

impl Display for Chip8Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LargeProgram => write!(f, "program too large for VM memory"),
        }
    }
}

impl std::error::Error for Chip8Error {}
