use std::error::Error;

#[macro_use]
extern crate slog;
use chip8_win::{Chip8App, InputMap};
use log::{error, info};
use slog::Drain;

fn main() -> Result<(), Box<dyn Error>> {
    let decorator = slog_term::PlainDecorator::new(std::io::stdout());
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!("version" => "0.5"));

    let _scope_guard = slog_scope::set_global_logger(logger);
    let _log_guard = slog_stdlog::init_with_level(log::Level::Trace).unwrap();

    info!("starting...");

    // Load input configuration
    let input_map = InputMap::from_file("chip8-win/input.yaml")?;
    log::debug!("loaded input map");

    let mut app = Chip8App::new(input_map)?;

    app.load_rom("chip8/programs/maze")?;

    match app.run() {
        Ok(_) => {}
        Err(err) => {
            error!("{err}");
            std::process::exit(1);
        }
    }

    info!("done");

    Ok(())
}
