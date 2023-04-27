mod app;
mod error;
mod inputmap;
mod render;
mod window;

/// Hardcoded input action names.
pub(crate) mod actions {
    /// Open or close the dev console
    pub const DEV_CONSOLE: &str = "devconsole";
    /// Exit the application
    pub const EXIT: &str = "exit";
    /// Reset the VM and reload the ROM
    pub const RESET: &str = "reset";
}

pub type EventLoop = winit::event_loop::EventLoop<()>;

pub use self::{
    app::{AppControl, Chip8App},
    error::{AppError, ErrorKind},
    inputmap::{InputKind, InputMap},
    window::WindowContext,
};

pub fn run_chip8_window(rom: &[u8], input_map: InputMap) -> Result<(), AppError> {
    log::info!("creating chip8 main window...");

    // Event loop can only be created once per process.
    let mut event_loop = Chip8App::create_event_loop();
    let window_ctx = WindowContext::new(&event_loop);
    let mut app = Chip8App::from_window(window_ctx, input_map);

    loop {
        app.load_rom_bytecode(rom)?;

        if let AppControl::Exit = app.run(&mut event_loop)? {
            break;
        }
    }

    log::info!("closed chip8 main window");
    Ok(())
}
