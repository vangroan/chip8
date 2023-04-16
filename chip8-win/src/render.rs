use std::rc::Rc;
use std::{fmt, marker::PhantomData};

use chip8::constants::{DISPLAY_BUFFER_SIZE, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use chip8::Chip8DisplayBuffer;
use glow::{Context as GlowContext, HasContext};
use winit::dpi::PhysicalSize;

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
    chip8_display: Chip8Display,
    framebuffer: Framebuffer,
    shader: ShaderProgram,
}

impl Render {
    pub fn new(gl: Rc<GlowContext>) -> Self {
        let info = OpenGLInfo::new(gl.as_ref());
        let chip8_display = Self::create_chip8_points(gl.as_ref());
        let framebuffer = Self::create_framebuffer(gl.as_ref());
        let shader = Self::compile_shaders(gl.as_ref());
        Self {
            gl,
            info,
            chip8_display,
            framebuffer,
            shader,
        }
    }

    fn create_framebuffer(gl: &GlowContext) -> Framebuffer {
        log::debug!("creating framebuffer");
        let width = 800;
        let height = 600;
        let size = PhysicalSize::new(width, height).cast();

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

            Framebuffer {
                size,
                fbo,
                tex,
                rbo,
            }
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

            // Determine uniform locations
            let mut uniforms = vec![];
            if let Some(u_color_loc) = gl.get_uniform_location(program, "u_Color") {
                uniforms.push(("u_Color", u_color_loc));
            }

            ShaderProgram {
                prog: program,
                uniforms: uniforms.into_boxed_slice(),
            }
        }
    }

    fn create_chip8_points(gl: &GlowContext) -> Chip8Display {
        // Points describing the pixels on the Chip8 display
        let points = &mut [Point::default(); DISPLAY_BUFFER_SIZE];
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                points[x + y * DISPLAY_WIDTH] = Point {
                    position: [x as f32, y as f32],
                    alpha: 0.0,
                    ..Point::default()
                };
            }
        }

        // Primitive type points, not triangles
        let indices = &mut [0_u16; DISPLAY_BUFFER_SIZE];
        for index in 0..indices.len() {
            indices[index] = index as u16;
        }

        unsafe {
            // ================================================================
            // Vertex Array Object
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));

            // ================================================================
            // Vertex Buffer OBject
            let vertex_buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(points),
                glow::DYNAMIC_DRAW,
            );
            gl_error!(gl);

            // ----------------------------------------------------------------
            // Vertex data is interleaved.
            // Attribute layout positions are determined by shader.
            // Positions
            gl.enable_vertex_attrib_array(Point::POSITION_LOC);
            gl.vertex_attrib_pointer_f32(
                Point::POSITION_LOC,                 // Attribute location in shader program.
                2,                                   // Size. Components per iteration.
                glow::FLOAT,                         // Type to get from buffer.
                false,                               // Normalize.
                std::mem::size_of::<Point>() as i32, // Stride. Bytes to advance each iteration.
                memoffset::offset_of!(Point, position) as i32, // Offset. Bytes from start of buffer.
            );
            gl_error!(gl);

            // ----------------------------------------------------------------
            // Chip8 Pixel Alpha
            gl.enable_vertex_attrib_array(Point::ALPHA_LOC);
            gl.vertex_attrib_pointer_f32(
                Point::ALPHA_LOC,                           // Attribute location in shader program.
                1,                                          // Size. Components per iteration.
                glow::FLOAT,                                // Type to get from buffer.
                false,                                      // Normalize.
                std::mem::size_of::<Point>() as i32, // Stride. Bytes to advance each iteration.
                memoffset::offset_of!(Point, alpha) as i32, // Offset. Bytes from start of buffer.
            );
            gl_error!(gl);

            // ----------------------------------------------------------------
            // Index Buffer
            let index_buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(indices),
                glow::STATIC_DRAW,
            );
            gl_error!(gl);

            // ----------------------------------------------------------------
            // Unbind
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            Chip8Display {
                points: Box::new(points.clone()),
                vertex_array: VertexArray {
                    vao,
                    vertex_buffer,
                    index_buffer,
                    _vertex: PhantomData,
                },
            }
        }
    }

    pub fn draw_chip8_display(&mut self, chip8_buf: Chip8DisplayBuffer) {
        assert_eq!(chip8_buf.len(), self.chip8_display.points.len());

        // Build points from given buffer
        for index in 0..chip8_buf.len() {
            let pixel_state = chip8_buf[index];
            self.chip8_display.points[index].alpha = if pixel_state { 1.0 } else { 0.0 };
        }

        unsafe {
            // TODO: Change to render to texture
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            self.gl.use_program(Some(self.shader.prog));
            self.gl
                .bind_vertex_array(Some(self.chip8_display.vertex_array.vao));
            let u_color_loc = self.shader.uniform_location("u_Color");
            assert!(u_color_loc.is_some());
            self.gl.uniform_4_f32(u_color_loc, 0.8, 0.9, 1.0, 1.0);

            self.gl
                .draw_arrays(glow::POINTS, 0, self.chip8_display.points.len() as i32);

            self.gl.bind_vertex_array(None);
            self.gl.use_program(None);
            gl_error!(self.gl);
        }
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
            let vertex_array = &self.chip8_display.vertex_array;

            log::debug!("deleting vertex buffer: {:?}", vertex_array.vertex_buffer);
            gl.delete_buffer(vertex_array.vertex_buffer);

            log::debug!("deleting index buffer: {:?}", vertex_array.index_buffer);
            gl.delete_buffer(vertex_array.index_buffer);

            log::debug!("deleting vertex array: {:?}", vertex_array.vao);
            gl.delete_vertex_array(vertex_array.vao);

            log::debug!("deleting render texture: {:?}", self.framebuffer.tex);
            gl.delete_texture(self.framebuffer.tex);

            log::debug!("deleting render buffer: {:?}", self.framebuffer.rbo);
            gl.delete_renderbuffer(self.framebuffer.rbo);

            log::debug!("deleting frame buffer: {:?}", self.framebuffer.fbo);
            gl.delete_framebuffer(self.framebuffer.fbo);

            log::debug!("deleting shader program: {:?}", self.shader.prog);
            gl.delete_program(self.shader.prog);
        }
    }
}

struct Framebuffer {
    size: PhysicalSize<u32>,
    fbo: glow::NativeFramebuffer,
    tex: glow::Texture,
    rbo: glow::Renderbuffer,
}

struct ShaderProgram {
    prog: glow::NativeProgram,
    uniforms: Box<[(&'static str, glow::NativeUniformLocation)]>,
}

impl ShaderProgram {
    fn uniform_location(&self, name: &str) -> Option<&glow::NativeUniformLocation> {
        self.uniforms
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, l)| l)
    }
}

/// Vertex points that represent the pixels on the Chip8 display.
struct Chip8Display {
    points: Box<[Point; DISPLAY_BUFFER_SIZE]>,
    vertex_array: VertexArray<Point>,
}

struct VertexArray<T> {
    vao: glow::NativeVertexArray,
    vertex_buffer: glow::NativeBuffer,
    index_buffer: glow::NativeBuffer,
    _vertex: PhantomData<T>,
}

#[derive(Default, bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C)]
struct Point {
    position: [f32; 2],
    alpha: f32,
    _pad: u32,
}

impl Point {
    const POSITION_LOC: u32 = 0;
    const ALPHA_LOC: u32 = 1;
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
