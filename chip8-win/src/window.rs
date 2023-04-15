use std::num::NonZeroU32;
use std::rc::Rc;

use glutin::config::Config as GlutinConfig;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::GlProfile;
use glutin::context::{ContextApi, ContextAttributesBuilder, Version as GlVersion};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::GlSurface;
use glutin::surface::SwapInterval;
use glutin::surface::WindowSurface;
use glutin_winit::GlWindow;
use raw_window_handle::HasRawWindowHandle;
use winit::dpi::LogicalSize;
use winit::dpi::PhysicalSize;
use winit::window::WindowBuilder;

use crate::EventLoop;

#[allow(dead_code)]
pub struct WindowContext {
    pub(crate) window: winit::window::Window,
    pub(crate) gl_context: glutin::context::PossiblyCurrentContext,
    pub(crate) gl_display: glutin::display::Display,
    pub(crate) gl_surface: glutin::surface::Surface<WindowSurface>,
    pub(crate) gl: Rc<glow::Context>,
}

impl WindowContext {
    /// Create a Window with an OpenGL context.
    ///
    /// - For Windows, the main window must be created first, for the OpenGL
    ///   context to be created.
    /// - For Android, the OpenGL context is created before the window exists.
    pub fn new(event_loop: &EventLoop) -> Self {
        // --------------------------------------------------------------------
        // Window

        let inner_size = LogicalSize::new(640, 480);
        let window_builder = WindowBuilder::new()
            .with_resizable(true)
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
            .prefer_hardware_accelerated(Some(true))
            .with_alpha_size(8)
            .with_transparency(cfg!(cgl_backend));

        // Helper crate handles the cross-platform complexity of setting up an OpenGL context.
        let (window, gl_config) = glutin_winit::DisplayBuilder::new()
            .with_preference(glutin_winit::ApiPrefence::FallbackEgl)
            .with_window_builder(Some(window_builder.clone()))
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
        let window = window.unwrap_or_else(|| {
            log::info!("creating window with finalize_window");
            glutin_winit::finalize_window(event_loop, window_builder.clone(), &gl_config)
                .expect("failed to finalize window")
        });

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
        log::debug!("attempt to set vsync");
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

        let gl = unsafe {
            Rc::new(glow::Context::from_loader_function_cstr(|symbol| {
                gl_display.get_proc_address(symbol)
            }))
        };

        Self {
            window,
            gl_context,
            gl_display,
            gl_surface,
            gl,
        }
    }

    /// Returns an identifier unique to the window.
    #[inline]
    pub fn window_id(&self) -> winit::window::WindowId {
        self.window.id()
    }

    /// Emits a [`Event::RedrawRequested`] event in the associated event loop after all OS
    /// events have been processed by the event loop.
    ///
    /// This is the **strongly encouraged** method of redrawing windows, as it can integrate with
    /// OS-requested redraws (e.g. when a window gets resized).
    ///
    /// This function can cause `RedrawRequested` events to be emitted after [`Event::MainEventsCleared`]
    /// but before `Event::NewEvents` if called in the following circumstances:
    /// * While processing `MainEventsCleared`.
    /// * While processing a `RedrawRequested` event that was sent during `MainEventsCleared` or any
    ///   directly subsequent `RedrawRequested` event.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Can only be called on the main thread.
    /// - **Android:** Subsequent calls after `MainEventsCleared` are not handled.
    ///
    /// [`Event::RedrawRequested`]: crate::event::Event::RedrawRequested
    /// [`Event::MainEventsCleared`]: crate::event::Event::MainEventsCleared
    #[inline]
    pub fn request_redraw(&self) {
        self.window.request_redraw()
    }

    /// Swaps the underlying back buffers when the surface is not single buffered.
    #[inline]
    #[must_use]
    pub fn swap_buffers(&self) -> glutin::error::Result<()> {
        self.gl_surface.swap_buffers(&self.gl_context)
    }

    /// Make the underlying surface current on the calling thread.
    #[inline]
    #[must_use]
    pub fn make_context_current(&self) -> glutin::error::Result<()> {
        self.gl_context.make_current(&self.gl_surface)
    }

    /// Resize the surface to a new size.
    ///
    /// Does not resize the window.
    ///
    /// This call is for compatibility reasons, on most platforms it's a no-op.
    pub fn resize_surface(&self, size: impl Into<PhysicalSize<u32>>) {
        let size = size.into();
        // Zero sized surface is invalid.
        if size.width != 0 && size.height != 0 {
            self.gl_surface.resize(
                &self.gl_context,
                NonZeroU32::new(size.width).unwrap(),
                NonZeroU32::new(size.height).unwrap(),
            );
            // TODO: Resize OpenGL viewport.
        }
    }
}
