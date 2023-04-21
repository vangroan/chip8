//! Application errors
use std::fmt;

#[derive(Debug)]
pub struct AppError {
    pub kind: ErrorKind,
}

impl std::error::Error for AppError {}

#[derive(Debug)]
pub enum ErrorKind {
    Chip8(chip8::Chip8Error),
    Io(std::io::Error),
    Window(winit::error::OsError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "application error: {}", self.kind)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Chip8(err) => write!(f, "{err}"),
            Self::Io(err) => write!(f, "{err}"),
            Self::Window(err) => write!(f, "{err}"),
        }
    }
}

impl From<chip8::Chip8Error> for AppError {
    fn from(err: chip8::Chip8Error) -> Self {
        Self {
            kind: ErrorKind::Chip8(err),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Io(err),
        }
    }
}

impl From<winit::error::OsError> for AppError {
    fn from(err: winit::error::OsError) -> Self {
        Self {
            kind: ErrorKind::Window(err),
        }
    }
}
