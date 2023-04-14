use std::fmt;

use glow::{Context as GlowContext, HasContext};
use glutin::display::Display;
use glutin::prelude::GlDisplay;

pub struct Render {
    /// The interface to the loaded OpenGL function.
    gl: GlowContext,
    info: OpenGLInfo,
}

impl Render {
    pub fn new(gl_display: &Display) -> Self {
        // Create glow context.
        let gl = unsafe {
            glow::Context::from_loader_function_cstr(|symbol| gl_display.get_proc_address(symbol))
        };

        let info = OpenGLInfo::new(&gl);

        Self { gl, info }
    }

    fn create_buffers(_gl: &GlowContext) {
        todo!("create vertex buffer and frame buffer")
    }

    pub fn clear_window(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        unsafe {
            self.gl.clear_color(red, green, blue, alpha);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    pub fn opengl_info(&self) -> &OpenGLInfo {
        &self.info
    }
}

pub struct OpenGLInfo {
    pub version: String,
    pub renderer: String,
    pub vendor: String,
    pub shading_lang: String,
}

impl OpenGLInfo {
    pub fn new(gl: &GlowContext) -> Self {
        unsafe {
            Self {
                version: gl.get_parameter_string(glow::VERSION),
                renderer: gl.get_parameter_string(glow::RENDERER),
                vendor: gl.get_parameter_string(glow::VENDOR),
                shading_lang: gl.get_parameter_string(glow::SHADING_LANGUAGE_VERSION),
            }
        }
    }
}

impl fmt::Display for OpenGLInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            version,
            renderer,
            vendor,
            shading_lang,
        } = self;
        writeln!(f, "OpenGL Version: {version}")?;
        writeln!(f, "Renderer: {renderer}")?;
        writeln!(f, "Vendor: {vendor}")?;
        writeln!(f, "Shading Language: {shading_lang}")?;
        Ok(())
    }
}
