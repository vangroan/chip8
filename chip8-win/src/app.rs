use std::io::Read;

use chip8::prelude::*;
use log::info;
use winit::{
    dpi::LogicalSize,
    event::{Event as EV, WindowEvent as WE},
    event_loop::{EventLoop, EventLoopBuilder},
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
};

use crate::error::AppError;

/// Chip8 Application
pub struct Chip8App {
    window: Window,
    event_loop: EventLoop<()>,
    vm: Chip8Vm,
}

impl Chip8App {
    pub fn new() -> Result<Self, AppError> {
        let inner_size = LogicalSize::new(640, 480);

        let event_loop = EventLoopBuilder::new().build();
        let window = WindowBuilder::new()
            .with_inner_size(inner_size)
            .with_title("chip8")
            .build(&event_loop)?;

        let vm = Chip8Vm::new(Chip8Conf {
            clock_frequency: None,
        });

        Ok(Self {
            window,
            event_loop,
            vm,
        })
    }

    /// Load rom file into VM
    pub fn load_rom(&mut self, filepath: &str) -> Result<(), AppError> {
        info!("load rom: {filepath}");

        let mut buf = vec![];

        let mut file = std::fs::File::open(filepath)?;
        file.read_to_end(&mut buf)?;

        self.vm.load_bytecode(&buf)?;

        Ok(())
    }
}

/// Event Loop.
impl Chip8App {
    pub fn run(&mut self) -> Result<(), AppError> {
        let main_window_id = self.window.id();

        self.event_loop.run_return(|event, _, control_flow| {
            control_flow.set_poll();

            match event {
                EV::NewEvents(_) => {
                    // Frame start
                }
                EV::MainEventsCleared => {
                    // Update

                    self.vm.execute().expect("vm error");

                    self.window.request_redraw();
                }
                EV::RedrawRequested(_) => {
                    // TODO: Render
                }
                EV::WindowEvent { window_id, event } if window_id == main_window_id => {
                    match event {
                        WE::CloseRequested => {
                            control_flow.set_exit();
                        }
                        WE::Resized(_new_size) => {}
                        _ => { /* blank */ }
                    }
                }
                _ => { /* blank */ }
            }
        });

        Ok(())
    }
}
