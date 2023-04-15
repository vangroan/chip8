use std::io::Read;

use chip8::prelude::*;
use log::info;
use winit::{
    event::{Event as EV, WindowEvent as WE},
    event_loop::EventLoopBuilder,
    platform::run_return::EventLoopExtRunReturn,
};

use crate::{
    actions::*, error::AppError, render::Render, window::WindowContext, EventLoop, InputMap,
};

/// Chip8 Application
pub struct Chip8App {
    window_ctx: WindowContext,
    render: Render,
    vm: Chip8Vm,
    input_map: InputMap,
}

impl Chip8App {
    /// Create the Chip8 window app.
    pub fn from_window(window_ctx: WindowContext, input_map: InputMap) -> Self {
        // Create an application specific renderer.
        let render = Render::new(window_ctx.gl.clone());
        log::info!("OpenGL renderer created:\n{}", render.opengl_info());

        // Create Chip8 emulated
        let vm = Chip8Vm::new(Chip8Conf {
            clock_frequency: None,
        });

        Self {
            window_ctx,
            render,
            input_map,
            vm,
        }
    }

    pub fn create_event_loop() -> EventLoop {
        EventLoopBuilder::new().build()
    }

    /// Load ROM file into VM
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
    pub fn run(&mut self, event_loop: &mut EventLoop) -> Result<(), AppError> {
        let main_window_id = self.window_ctx.window.id();

        event_loop.run_return(|event, _, control_flow| {
            control_flow.set_poll();

            match event {
                EV::NewEvents(_) => {
                    // Frame start
                    self.input_map.clear_state();
                }
                EV::MainEventsCleared => {
                    // Frame Update

                    if self.input_map.is_action_pressed(DEV_CONSOLE) {
                        log::info!("Developer Console");
                    }

                    if self.input_map.is_action_pressed(EXIT) {
                        log::info!("Exit");
                        control_flow.set_exit();
                    }

                    // Merge input stream into VM
                    for keycode in self.input_map.iter_chip8() {
                        self.vm.set_key(keycode, true);
                    }

                    let s = self.vm.dump_keys().unwrap();
                    if !s.is_empty() {
                        log::debug!("{s}");
                    }

                    // TODO: graceful error handling
                    self.vm.tick().unwrap();

                    // Queue a RedrawRequested event.
                    //
                    // You only need to call this if you've determined that you need to redraw, in
                    // applications which do not always need to. Applications that redraw continuously
                    // can just render here instead.
                    self.window_ctx.request_redraw();
                }
                EV::RedrawRequested(_) => {
                    // Redraw the application.
                    if let Ok(_) = self.window_ctx.make_context_current() {
                        self.render
                            .clear_window(29.0 / 255.0, 33.0 / 255.0, 40.0 / 255.0, 0.9);

                        self.window_ctx.swap_buffers().unwrap();
                    }
                }
                EV::WindowEvent { window_id, event } if window_id == main_window_id => {
                    match event {
                        WE::Resized(size) => {
                            // Some platforms like EGL require resizing GL surface to update the size.
                            // Notable platforms here are Wayland and macOS, others don't require it
                            // and the function is no-op, but it's wise to resize it for portability
                            // reasons.
                            // Zero sized surface is invalid.
                            self.window_ctx.resize_surface(size);
                        }
                        WE::KeyboardInput { input, .. } => {
                            if let Some(virtual_keycode) = input.virtual_keycode {
                                self.input_map.push_key(virtual_keycode, input.state);
                            }
                        }
                        WE::CloseRequested => {
                            control_flow.set_exit();
                        }
                        _ => { /* blank */ }
                    }
                }
                _ => { /* blank */ }
            }
        });

        Ok(())
    }
}
