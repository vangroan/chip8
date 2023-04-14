mod app;
mod error;
mod inputmap;

/// Hardcoded input action names.
pub(crate) mod actions {
    /// Open or close the dev console
    pub const DEV_CONSOLE: &str = "devconsole";
    /// Exit the application
    pub const EXIT: &str = "exit";
}

pub type EventLoop = winit::event_loop::EventLoop<()>;

pub use self::{
    app::Chip8App,
    error::{AppError, ErrorKind},
    inputmap::{InputKind, InputMap},
};
