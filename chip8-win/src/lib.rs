mod app;
mod error;

pub use self::{
    app::Chip8App,
    error::{AppError, ErrorKind},
};
