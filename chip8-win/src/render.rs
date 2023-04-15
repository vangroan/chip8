use std::fmt;
use std::rc::Rc;

use chip8::constants::{DISPLAY_BUFFER_SIZE, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use glow::{Context as GlowContext, HasContext};

macro_rules! gl_error {
    ($gl:expr) => {
        #[cfg(debug_assertions)]
        {
            let line = line!();
            let file = file!();
            let _: &glow::Context = &$gl; // type assert
            let mut has_error = false;
            loop {
                let err = $gl.get_error();
                if err == glow::NO_ERROR {
                    break;
                }
                has_error = true;
                log::error!("OpenGL error [{file}:{line}]: 0x{err:04x}");
            }
            if has_error {
                panic!("OpenGL Errors. See logs.");
            }
        }
    };
}

macro_rules! shader_error {
    ($gl:expr, $shader:expr, $name:expr) => {{
        let line = line!();
        let file = file!();
        let _: &glow::Context = &$gl; // type assert
        let _: &glow::NativeShader = &$shader;
        if !$gl.get_shader_compile_status($shader) {
            log::error!(
                "failed to compile {} [{file}:{line}]: {}",
                $name,
                $gl.get_shader_info_log($shader)
            );
            panic!("shader compilation error");
        }
    }};
}

pub struct Render {
    /// The interface to the loaded OpenGL function.
    gl: Rc<GlowContext>,
    info: OpenGLInfo,
    framebuffer: Framebuffer,
    shader: ShaderProgram,
}

impl Render {
    pub fn new(gl: Rc<GlowContext>) -> Self {
        let info = OpenGLInfo::new(gl.as_ref());
        let framebuffer = Self::create_framebuffer(gl.as_ref());
        let shader = Self::compile_shaders(gl.as_ref());
        Self {
            gl,
            info,
            framebuffer,
            shader,
        }
    }

    fn create_framebuffer(gl: &GlowContext) -> Framebuffer {
        log::debug!("creating framebuffer");
        let width = 800;
        let height = 600;

        unsafe {
            let fbo = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl_error!(gl);

            // ----------------------------------------------------------------
            // Colour Attachment
            let tex = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            // gl.bind_texture(glow::TEXTURE_2D, None);

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                width,
                height,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                None,
            );
            gl_error!(gl);

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl_error!(gl);

            // ----------------------------------------------------------------
            // Depth and Stencil
            let rbo = gl.create_renderbuffer().unwrap();
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rbo));
            gl.renderbuffer_storage(glow::RENDERBUFFER, glow::DEPTH24_STENCIL8, width, height);
            gl.bind_renderbuffer(glow::RENDERBUFFER, None);
            gl_error!(gl);

            // ----------------------------------------------------------------
            // Attach
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(tex),
                0,
            );
            gl.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::DEPTH_STENCIL_ATTACHMENT,
                glow::RENDERBUFFER,
                Some(rbo),
            );
            assert!(
                gl.check_framebuffer_status(glow::FRAMEBUFFER) == glow::FRAMEBUFFER_COMPLETE,
                "framebuffer is not complete"
            );
            gl_error!(gl);

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            Framebuffer { fbo, tex, rbo }
        }
    }

    fn compile_shaders(gl: &GlowContext) -> ShaderProgram {
        log::debug!("compiling shaders");
        unsafe {
            let vert_shader = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            gl.shader_source(vert_shader, include_str!("shader_chip8.vert"));
            gl.compile_shader(vert_shader);
            shader_error!(gl, vert_shader, "vertex shader");

            let geom_shader = gl.create_shader(glow::GEOMETRY_SHADER).unwrap();
            gl.shader_source(geom_shader, include_str!("shader_chip8.geom"));
            gl.compile_shader(geom_shader);
            shader_error!(gl, vert_shader, "geometry shader");

            let frag_shader = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(frag_shader, include_str!("shader_chip8.frag"));
            gl.compile_shader(frag_shader);
            shader_error!(gl, vert_shader, "fragment shader");

            let program = gl.create_program().unwrap();
            gl.attach_shader(program, vert_shader);
            gl.attach_shader(program, geom_shader);
            gl.attach_shader(program, frag_shader);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                let message = gl.get_program_info_log(program);
                log::error!("failed to link shader program: {message}");
            }

            // Flag the shader objects for deletion. They will be deleted later
            // automatically when they're detached from the shader program.
            gl.delete_shader(vert_shader);
            gl.delete_shader(geom_shader);
            gl.delete_shader(frag_shader);

            ShaderProgram(program)
        }
    }

    fn create_points(gl: &GlowContext) {
        unsafe {
            let vertex_buf = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buf));
            gl.buffer_storage(
                glow::ARRAY_BUFFER,
                (DISPLAY_BUFFER_SIZE * std::mem::size_of::<Point>()) as i32,
                None,
                glow::DYNAMIC_DRAW,
            );
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl_error!(gl);
        }
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

impl Drop for Render {
    fn drop(&mut self) {
        let gl = self.gl.as_ref();

        unsafe {
            log::debug!("deleting render texture: {:?}", self.framebuffer.tex);
            gl.delete_texture(self.framebuffer.tex);

            log::debug!("deleting render buffer: {:?}", self.framebuffer.rbo);
            gl.delete_renderbuffer(self.framebuffer.rbo);

            log::debug!("deleting frame buffer: {:?}", self.framebuffer.fbo);
            gl.delete_framebuffer(self.framebuffer.fbo);

            log::debug!("deleting shader program: {:?}", self.shader.0);
            gl.delete_program(self.shader.0);
        }
    }
}

struct Framebuffer {
    fbo: glow::NativeFramebuffer,
    tex: glow::Texture,
    rbo: glow::Renderbuffer,
}

struct ShaderProgram(glow::NativeProgram);

#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C)]
struct Point {
    position: [f32; 2],
    alpha: f32,
    _pad: u32,
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
