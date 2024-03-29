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
    demo_pattern: Box<[bool; DISPLAY_BUFFER_SIZE]>,
}

impl Render {
    pub fn new(gl: Rc<GlowContext>) -> Self {
        let info = OpenGLInfo::new(gl.as_ref());
        let chip8_display = Self::create_chip8_display(gl.as_ref());
        let framebuffer = Self::create_framebuffer(gl.as_ref());
        Self {
            gl,
            info,
            chip8_display,
            framebuffer,
            demo_pattern: demo_display_pattern(),
        }
    }

    fn create_framebuffer(gl: &GlowContext) -> Framebuffer {
        log::debug!("creating framebuffer");
        let width = 800;
        let height = 400;
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
            assert_eq!(
                gl.check_framebuffer_status(glow::FRAMEBUFFER),
                glow::FRAMEBUFFER_COMPLETE,
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
            gl.shader_source(vert_shader, include_str!("shaders/chip8.vert"));
            gl.compile_shader(vert_shader);
            shader_error!(gl, vert_shader, "vertex shader");

            let geom_shader = gl.create_shader(glow::GEOMETRY_SHADER).unwrap();
            gl.shader_source(geom_shader, include_str!("shaders/chip8.geom"));
            gl.compile_shader(geom_shader);
            shader_error!(gl, vert_shader, "geometry shader");

            let frag_shader = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(frag_shader, include_str!("shaders/chip8.frag"));
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
            if let Some(u_matrix_loc) = gl.get_uniform_location(program, "u_Matrix") {
                uniforms.push(("u_Matrix", u_matrix_loc));
            } else {
                log::warn!("could not get location of uniform 'u_Matrix'");
            }

            ShaderProgram {
                prog: program,
                uniforms: uniforms.into_boxed_slice(),
            }
        }
    }

    fn create_chip8_display(gl: &GlowContext) -> Chip8Display {
        let shader = Self::compile_shaders(gl);

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
        for (index, item) in indices.iter_mut().enumerate() {
            *item = index as u16;
        }

        unsafe {
            // ================================================================
            // Vertex Array Object
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));

            // ================================================================
            // Vertex Buffer Object
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

            // ================================================================
            // Unbind
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            // ================================================================
            // Chip8 Matrix
            let matrix = flatten_matrix(&Chip8Display::matrix());

            Chip8Display {
                shader,
                points: Box::new(*points),
                vertex_array: VertexArray {
                    vao,
                    vertex_buffer,
                    index_buffer,
                    _vertex: PhantomData,
                },
                matrix,
            }
        }
    }

    pub fn draw_chip8_display(&mut self, chip8_buf: Chip8DisplayBuffer) {
        self.chip8_display.copy_points(chip8_buf);
        self.chip8_display.draw(&self.gl);
    }

    /// Draw a test pattern.
    ///
    /// Useful for checking the correctness of the
    /// render pipeline and shader program.
    #[allow(dead_code)]
    pub fn draw_demo_pattern(&mut self) {
        self.chip8_display.copy_points(&self.demo_pattern);
        self.chip8_display.draw(&self.gl);
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
            let Chip8Display {
                shader,
                vertex_array,
                ..
            } = &self.chip8_display;

            vertex_array.delete(gl);
            shader.delete(gl);
            self.framebuffer.delete(gl);
        }
    }
}

#[rustfmt::skip]
pub const DEMO_DISPLAY: &[u32; 64] = &[
    0xFF008888, 0x888888FF,  // 0
    0x81444444, 0x44444481,  // 1
    0x99282222, 0x222222BD,  // 2
    0xBD101111, 0x111110A5,  // 3
    0xBD288888, 0x888888A5,  // 4
    0x99444444, 0x444444BD,  // 5
    0x81442222, 0x22222281,  // 6
    0xFF001111, 0x111110FF,  // 7
    0x00000000, 0x00000000,  // 8
    0x22000000, 0x00000000,  // 9
    0x22000000, 0x00000000,  // 10
    0x14000000, 0x00000000,  // 11
    0x08000000, 0x00000000,  // 12
    0x08000000, 0x00000000,  // 13
    0x08000000, 0x00000000,  // 14
    0x00000000, 0x00000000,  // 15
    0x00000000, 0x00000000,  // 16
    0x00000000, 0x00000000,  // 17
    0x00000000, 0x00000000,  // 18
    0x00000000, 0x00000000,  // 19
    0x00000000, 0x00000000,  // 20
    0x00000000, 0x00000000,  // 21
    0x00000000, 0x00000000,  // 22
    0x00000000, 0x00000000,  // 23
    0xFF000000, 0x000000FF,  // 24
    0x81000000, 0x00000081,  // 25
    0x99000000, 0x000000BD,  // 26
    0xBD000000, 0x000000BD,  // 27
    0xBD000000, 0x000000BD,  // 28
    0x99000000, 0x000000BD,  // 29
    0x81000000, 0x00000081,  // 30
    0xFF000000, 0x000000FF,  // 31
];

#[allow(dead_code)]
pub fn demo_display_pattern() -> Box<[bool; DISPLAY_BUFFER_SIZE]> {
    let buf = &mut [false; DISPLAY_BUFFER_SIZE];
    const U32_BITS: usize = u32::BITS as usize;

    for y in 0..DISPLAY_HEIGHT {
        for x in 0..DISPLAY_WIDTH {
            let dst_index = x + y * DISPLAY_WIDTH;
            let index_a = dst_index / U32_BITS;
            let index_b = U32_BITS - 1 - (dst_index % U32_BITS);
            // print!("|{dst_index} {index_a} {index_b}|");
            let bit = (DEMO_DISPLAY[index_a] >> index_b) & 1;
            // print!("{}", if bit == 1 { '#' } else { '.' });
            buf[dst_index] = bit == 1;
        }
        println!();
    }

    Box::new(*buf)
}

struct Framebuffer {
    #[allow(dead_code)]
    size: PhysicalSize<u32>,
    fbo: glow::NativeFramebuffer,
    tex: glow::Texture,
    rbo: glow::Renderbuffer,
}

impl Framebuffer {
    fn _new(_gl: &GlowContext, _size: PhysicalSize<u32>) -> Self {
        todo!("framebuffer as render texture target")
    }

    /// Delete inner resources.
    ///
    /// # Safety
    ///
    /// Attempting to use this resource after
    /// deletion will result in an OpenGL error.
    #[doc(hidden)]
    unsafe fn delete(&self, gl: &GlowContext) {
        gl.delete_texture(self.tex);
        gl.delete_renderbuffer(self.rbo);
        gl.delete_framebuffer(self.fbo);
    }
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

    /// Delete inner resources.
    ///
    /// # Safety
    ///
    /// Attempting to use this resource after
    /// deletion will result in an OpenGL error.
    #[doc(hidden)]
    unsafe fn delete(&self, gl: &GlowContext) {
        gl.delete_program(self.prog);
    }
}

/// Vertex points that represent the pixels on the Chip8 display.
struct Chip8Display {
    shader: ShaderProgram,
    points: Box<[Point; DISPLAY_BUFFER_SIZE]>,
    vertex_array: VertexArray<Point>,
    matrix: [f32; 16],
}

impl Chip8Display {
    fn copy_points(&mut self, chip8_buf: Chip8DisplayBuffer) {
        assert_eq!(chip8_buf.len(), self.points.len());

        // Build points from given buffer
        for (index, pixel_state) in chip8_buf.iter().enumerate() {
            self.points[index].alpha = if *pixel_state { 1.0 } else { 0.0 };
        }
    }

    fn draw(&self, gl: &GlowContext) {
        let Self {
            shader,
            points,
            vertex_array,
            matrix,
        } = self;

        unsafe {
            gl.disable(glow::CULL_FACE);
            gl.enable(glow::BLEND);

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            gl.use_program(Some(shader.prog));
            gl.bind_vertex_array(Some(vertex_array.vao));

            // Upload vertex data
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_array.vertex_buffer));
            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                bytemuck::cast_slice(points.as_slice()),
            );

            let u_color_loc = shader.uniform_location("u_Color");
            assert!(u_color_loc.is_some());
            gl.uniform_4_f32(u_color_loc, 0.8, 0.9, 1.0, 1.0);

            let u_matrix_loc = shader.uniform_location("u_Matrix");
            assert!(u_matrix_loc.is_some());
            assert_eq!(matrix.len(), 16);
            gl.uniform_matrix_4_f32_slice(
                u_matrix_loc,
                true,
                bytemuck::cast_slice(matrix.as_slice()),
            );

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(vertex_array.index_buffer));
            gl.draw_elements(glow::POINTS, points.len() as i32, glow::UNSIGNED_SHORT, 0);

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.use_program(None);
            gl_error!(gl);
        }
    }

    /// Create a view matrix from Chip8 display coordinates to OpenGL clip space.
    #[allow(unused_assignments)]
    #[rustfmt::skip]
    fn matrix() -> [[f32; 4]; 4] {
        let mut sx: f32 = 1.0;
        let mut sy: f32 = 1.0;
        let mut tx: f32 = 0.0;
        let mut ty: f32 = 0.0;

        // Normalize the vertex position from chip8 pixels to 0.0 to 1.0
        //
        // chip8_resolution = vec2(64, 32)
        // norm_position = 1 / chip8_resolution
        sx = 1.0 / DISPLAY_WIDTH as f32;
        sy = 1.0 / DISPLAY_HEIGHT as f32;

        // Convert from normalized position (0,+1) to clip space (-1,+1)
        //
        // clip_position = norm_position * 2 - 1
        sx *= 2.0;
        sy *= 2.0;
        tx = -1.0;
        ty = -1.0;

        // OpenGL y points upwards, for chip8 y points downwards.
        sy = -sy;
        ty = -ty; // translate after scale

        [
            [sx,  0.0, 0.0,  tx],
            [0.0,  sy, 0.0,  ty],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
}

struct VertexArray<T> {
    vao: glow::NativeVertexArray,
    vertex_buffer: glow::NativeBuffer,
    index_buffer: glow::NativeBuffer,
    _vertex: PhantomData<T>,
}

impl<T> VertexArray<T> {
    /// Delete inner resources.
    ///
    /// # Safety
    ///
    /// Attempting to use this resource after
    /// deletion will result in an OpenGL error.
    #[doc(hidden)]
    unsafe fn delete(&self, gl: &GlowContext) {
        gl.delete_buffer(self.vertex_buffer);
        gl.delete_buffer(self.index_buffer);
        gl.delete_vertex_array(self.vao);
    }
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

#[rustfmt::skip]
fn flatten_matrix(m: &[[f32; 4]; 4]) -> [f32; 16] {
    [
        m[0][0], m[0][1], m[0][2], m[0][3],
        m[1][0], m[1][1], m[1][2], m[1][3],
        m[2][0], m[2][1], m[2][2], m[2][3],
        m[3][0], m[3][1], m[3][2], m[3][3],
    ]
}
