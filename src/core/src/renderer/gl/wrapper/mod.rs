extern crate gl;

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::*;
use std::collections::HashMap;

use self::gl::types::*;

use math::Trans;

// pub use self::array_buffer::ArrayBuffer;
pub use self::program::Program;
pub use self::shader::Shader;
pub use self::texture::Texture;
// pub use self::vertex_array::VertexArray;

// mod array_buffer;
mod program;
mod shader;
mod texture;
// mod vertex_array;

use window::{Window, GlContext};
use Error;

#[derive(Clone)]
pub struct Context {
    raw: Rc<RefCell<GlContext>>,
    state: Rc<RefCell<State>>,
}

impl Context {
    pub fn new(window: &Window) -> Context {
        let mut context = window.create_gl_context();

        static OPENGL_FUNCTION_INIT: Once = ONCE_INIT;

        // TODO: Make sure that initialize OpenGL function once is enough.
        OPENGL_FUNCTION_INIT.call_once(|| {
            context.make_current();
            gl::load_with(|symbol| { context.load_function(symbol) });
        });

        Context {
            raw: Rc::new(RefCell::new(context)),
            state: Rc::new(RefCell::new(State::new())),
        }
    }

    pub fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        self.make_current();
        unsafe { gl::ClearColor(r, g, b, a); }
    }

    pub fn clear(&self) {
        self.make_current();
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); }
    }

    pub fn viewport(&self, x: i32, y: i32, w: i32, h: i32) {
        self.make_current();
        unsafe { gl::Viewport(x, y, w, h); }
    }

    pub fn active_texture(&self, texture: GLenum) {
        self.make_current();
        let mut state = self.state.borrow_mut();
        if state.active_texture != texture {
            unsafe { gl::ActiveTexture(texture); }
            state.active_texture = texture;
        }
    }

    pub fn bind_texture_2d(&self, id: GLuint) {
        self.make_current();
        let mut state = self.state.borrow_mut();
        let active_texture = state.active_texture;
        let texture_2d = state.texture_2d.entry(active_texture).or_insert(0);
        if *texture_2d != id {
            unsafe { gl::BindTexture(gl::TEXTURE_2D, id); }
            *texture_2d = id;
        }
    }

    pub fn bind_vertex_array(&self, id: GLuint) {
        self.make_current();
        let mut state = self.state.borrow_mut();
        if state.vertex_array != id {
            unsafe { gl::BindVertexArray(id); }
            state.vertex_array = id;
        }
    }

    pub fn bind_array_buffer(&self, id: GLuint) {
        self.make_current();
        let mut state = self.state.borrow_mut();
        if state.array_buffer != id {
            unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, id); }
            state.array_buffer = id;
        }
    }

    pub fn use_program(&self, id: GLuint) {
        self.make_current();
        let mut state = self.state.borrow_mut();
        if state.program != id {
            unsafe { gl::UseProgram(id); }
            state.program = id;
        }
    }

    pub fn swap_buffers(&self) {
        self.make_current();
        self.raw.borrow_mut().swap_buffers();
    }

    fn make_current(&self) {
        self.raw.borrow_mut().make_current();
    }
}

struct State {
    program: GLuint,
    active_texture: GLenum,
    texture_2d: HashMap<GLenum, GLuint>,
    array_buffer: GLuint,
    vertex_array: GLuint,
}

impl State {
    pub fn new() -> State {
        State {
            program: 0,
            active_texture: gl::TEXTURE0,
            texture_2d: HashMap::new(),
            array_buffer: 0,
            vertex_array: 0,
        }
    }
}

/*
pub enum BufferUsage {
    StaticDraw,
}

impl BufferUsage {
    pub fn to_gl(&self) -> GLenum {
        match *self {
            BufferUsage::StaticDraw => gl::STATIC_DRAW,
        }
    }
}
*/

pub struct Quad {
    context: Context,
    program: Program,
    vao: GLuint,
    _vbo: GLuint,
}

impl Quad {
    pub fn new(context: &Context) -> Result<Quad, Error> {
        const VERTEX_SHADER: &'static str = r#"
        #version 330 core

        uniform mat3 u_trans;

        layout (location = 0)
        in vec2 pos;

        layout (location = 1)
        in vec2 texcoord;

        out vec2 v_texcoord;

        void main() {
            gl_Position = vec4((u_trans * vec3(pos.xy, 1.0)).xy, 0.0, 1.0);
            v_texcoord = texcoord;
        }
        "#;

        const FRAGMENT_SHADER: &'static str = r#"
        #version 330 core

        uniform sampler2D u_texture0;

        in vec2 v_texcoord;

        out vec4 color;

        void main() {
            color = texture2D(u_texture0, v_texcoord);
        }
        "#;

        const VERTICES: [f32; 16] = [
            // Positions // Texture Coords
            0.0, 1.0,    0.0, 0.0,
            1.0, 1.0,    1.0, 0.0,
            0.0, 0.0,    0.0, 1.0,
            1.0, 0.0,    1.0, 1.0,
        ];

        let mut program = try!(Program::compile_and_link(context, VERTEX_SHADER, FRAGMENT_SHADER));

        program.set_uniform_1i("u_texture0", 0);

        /*
        let mut vbo = ArrayBuffer::new(context);
        vbo.buffer_data(Some(&VERTICES), BufferUsage::StaticDraw);

        let mut vao = VertexArray::new(context);
        vao.bind_array_buffer(&vbo);
        */

        use std::mem;
        use std::os::raw::c_void;

        unsafe {
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (VERTICES.len() * mem::size_of::<f32>()) as GLsizeiptr, VERTICES.as_ptr() as *const c_void, gl::STATIC_DRAW);

            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, (4 * mem::size_of::<GLfloat>()) as i32, 0 as *const c_void);
            gl::EnableVertexAttribArray(0);

            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (4 * mem::size_of::<GLfloat>()) as i32, (2 * mem::size_of::<GLfloat>()) as *const c_void);
            gl::EnableVertexAttribArray(1);

            gl::BindVertexArray(0);

            Ok(Quad {
                context: context.clone(),
                program: program,
                vao: vao,
                _vbo: vbo,
            })
        }
    }

    pub fn fill_with_texture(&mut self, window_to_clip_trans: Trans, x: f32, y: f32, w: f32, h: f32, texture: &Texture) {
        self.program.active();

        let trans = window_to_clip_trans * Trans::offset(x, y) * Trans::scale(w, h);
        let mat = trans.to_gl_mat3();
        self.program.set_uniform_matrix3_fv("u_trans", &mat);

        texture.active(0);
        self.context.bind_vertex_array(self.vao);
        unsafe { gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4); }
        self.context.bind_vertex_array(0);
    }
}
