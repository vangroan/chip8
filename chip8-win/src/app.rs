use std::{io::Read, num::NonZeroU32};

use chip8::prelude::*;
use glutin::{
    config::{Config as GlutinConfig, ConfigTemplateBuilder},
    context::{
        ContextApi, ContextAttributesBuilder, GlProfile, PossiblyCurrentContext,
        Version as GlVersion,
    },
    display::GetGlDisplay,
    prelude::*,
    surface::{Surface, SwapInterval, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use log::info;
use raw_window_handle::HasRawWindowHandle;
use winit::{
    dpi::LogicalSize,
    event::{Event as EV, WindowEvent as WE},
    event_loop::EventLoopBuilder,
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
};

use crate::{actions::*, error::AppError, render::Render, EventLoop, InputMap};

/// Chip8 Application
pub struct Chip8App {
    window: Window,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    render: Render,
    vm: Chip8Vm,
    input_map: InputMap,
}

impl Chip8App {
    /// Create the Chip8 window app.
    ///
    /// For Windows, the main window must be created first, for the OpenGL context to be created.
    ///
    /// For Android, the OpenGL context is created before the window exists.
    pub fn new(event_loop: &EventLoop, input_map: InputMap) -> Result<Self, AppError> {
        // --------------------------------------------------------------------
        // Window

        let inner_size = LogicalSize::new(640, 480);
        let window_builder = WindowBuilder::new()
            .with_inner_size(inner_size)
            .with_title("chip8")
            .with_transparent(true);

        // The template will match only the configurations supporting rendering
        // to windows.
        //
        // We force transparency only on macOS, given that EGL on X11 doesn't
        // have it, but we still want to show a window. The macOS situation is like
        // that, because we can query only one config at a time on it, but all
        // normal platforms will return multiple configs, so we can find the config
        // with transparency ourselves inside the `reduce`.
        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_transparency(cfg!(cgl_backend));

        let (window, gl_config) = DisplayBuilder::new()
            .with_window_builder(Some(window_builder))
            .build(&event_loop, template, |configs| {
                // Find a config that supports transparency, and has the maximum number of samples.
                let mut config: Option<GlutinConfig> = None;

                for c in configs {
                    if log::max_level() >= log::Level::Debug {
                        log::debug!(
                            "consider config: num_samples={}, supports_transparency={}",
                            c.num_samples(),
                            c.supports_transparency().unwrap_or(false)
                        );
                    }

                    // Does the next config support transparency?
                    let next_transparency = c.supports_transparency().unwrap_or(false);
                    // Does the previous config support transparency?
                    let prev_transparency = config
                        .as_ref()
                        .and_then(|config| config.supports_transparency())
                        .unwrap_or(false);

                    // Does the next config support transparency, but the previous config does not?
                    let supports_transparency = next_transparency && !prev_transparency;

                    // Does the next config have more samples than the previous config?
                    let more_samples = c.num_samples()
                        > config
                            .as_ref()
                            .map(|config| config.num_samples())
                            .unwrap_or(0);

                    if supports_transparency || more_samples {
                        config = Some(c);
                    }
                }

                config.expect("the system must supply at least one GL config")
            })
            .unwrap();

        if log::max_level() >= log::Level::Info {
            log::info!(
                "picked GL config with {} samples and {} transparency",
                gl_config.num_samples(),
                if gl_config.supports_transparency().unwrap_or(false) {
                    "with"
                } else {
                    "without"
                }
            );
        }

        // On Android, the window is not available when the OpenGL display has to be created.
        // However on Windows the main window must first exist before OpenGL can be initialized.
        let window = window.unwrap();

        // --------------------------------------------------------------------
        // OpenGL Context

        // Raw handle is required to build the OpenGL context.
        let raw_window_handle = window.raw_window_handle();

        // The display could be obtained from any object created by it, so we
        // can query it from the config.
        let gl_display = gl_config.display();

        // The context creation part. It can be created before surface and that's how
        // it's expected in multithreaded + multiwindow operation mode, since you
        // can *send* `NotCurrentContext`, but not `Surface`.
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(GlVersion::new(3, 3))))
            .with_profile(GlProfile::Core)
            .build(Some(raw_window_handle));

        // Since glutin by default tries to create OpenGL core context, which may not be
        // present we should try GLES.
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(Some(raw_window_handle));

        // Finally we can create the OpenGL context
        let not_current_gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    log::warn!("falling back to OpenGL ES");
                    gl_display
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("failed to create context")
                })
        };

        // --------------------------------------------------------------------
        // Surface

        // Create the main window surface
        let attrs = window.build_surface_attributes(<_>::default());
        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        // Make context current for the next phase of configuration.
        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

        // Attempt setting VSync
        if let Err(err) = gl_surface
            .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
        {
            log::error!("error setting vsync: {err:?}");
        }

        // --------------------------------------------------------------------
        // OpenGL Function Pointers

        // For WGL (Windows) the OpenGL context must be current,
        // otherwise only a subset of functions are loaded.
        if cfg!(wgl_backend) {
            assert!(
                gl_context.is_current(),
                "context must be current to load OpenGL functions"
            );
        }

        // Create renderer
        let render = Render::new(&gl_display);
        log::info!("Created OpenGL renderer:\n{}", render.opengl_info());

        // --------------------------------------------------------------------
        // Chip8 Virtual Machine

        let vm = Chip8Vm::new(Chip8Conf {
            clock_frequency: None,
        });

        Ok(Self {
            window,
            gl_context,
            gl_surface,
            render,
            vm,
            input_map,
        })
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
        let main_window_id = self.window.id();

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
                    self.window.request_redraw();
                }
                EV::RedrawRequested(_) => {
                    // Redraw the application.
                    if let Ok(_) = self.gl_context.make_current(&self.gl_surface) {
                        self.render
                            .clear_window(29.0 / 255.0, 33.0 / 255.0, 40.0 / 255.0, 0.9);

                        self.gl_surface.swap_buffers(&self.gl_context).unwrap();
                    }
                }
                EV::WindowEvent { window_id, event } if window_id == main_window_id => {
                    match event {
                        WE::Resized(size) => {
                            // Zero sized surface is invalid.
                            if size.width != 0 && size.height != 0 {
                                // Some platforms like EGL require resizing GL surface to update the size.
                                // Notable platforms here are Wayland and macOS, others don't require it
                                // and the function is no-op, but it's wise to resize it for portability
                                // reasons.
                                self.gl_surface.resize(
                                    &self.gl_context,
                                    NonZeroU32::new(size.width).unwrap(),
                                    NonZeroU32::new(size.height).unwrap(),
                                );
                                // TODO: Resize OpenGL viewport.
                            }
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
