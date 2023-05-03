use std::error::Error;

#[macro_use]
extern crate slog;
use chip8_win::{Chip8App, InputMap, WindowContext};
use log::{error, info};
use slog::Drain;

fn main() -> Result<(), Box<dyn Error>> {
    let decorator = slog_term::PlainDecorator::new(std::io::stdout());
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!("version" => "0.5"));

    let _scope_guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init_with_level(log::Level::Trace).unwrap();

    info!("starting...");

    // Load input configuration
    let input_map = InputMap::from_file("chip8-win/input.yaml")?;
    log::debug!("loaded input map");

    // Event loop can only be created once per process.
    let mut event_loop = Chip8App::create_event_loop();
    let window_ctx = WindowContext::new(&event_loop);
    let mut app = Chip8App::from_window(window_ctx, input_map);

    // app.load_rom_file("chip8/programs/maze")?;
    // app.load_rom_file("chip8/programs/BREAKOUT")?;
    app.load_rom_file("chip8/programs/TETRIS.ch8")?;
    // if let Err(err) = app.load_rom_asm(include_str!("../../programs/collision_test.asm")) {
    //     panic!("Failed to assemble program: {err}");
    // }
    // if let Err(err) = app.load_rom_asm(include_str!("../../programs/fontset_test.asm")) {
    //     panic!("Failed to assemble program: {err}");
    // }

    match app.run(&mut event_loop) {
        Ok(_) => {}
        Err(err) => {
            error!("{err}");
            std::process::exit(1);
        }
    }

    info!("done");

    Ok(())
}
