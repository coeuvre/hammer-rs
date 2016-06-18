extern crate gl;

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::*;

use self::gl::types::*;

pub use self::program::Program as Program;
pub use self::shader::Shader as Shader;
pub use self::texture::Texture as Texture;

mod program;
mod shader;
mod texture;

use window::{Window, GlContext};

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

    pub fn bind_texture_2d(&self, id: GLuint) {
        let mut state = self.state.borrow_mut();
        if state.current_texture_2d != id {
            unsafe { gl::BindTexture(gl::TEXTURE_2D, id); }
            state.current_texture_2d = id;
        }
    }

    pub fn use_program(&self, id: GLuint) {
        let mut state = self.state.borrow_mut();
        if state.current_program != id {
            unsafe { gl::UseProgram(id); }
            state.current_program = id;
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
    current_program: GLuint,
    current_texture_2d: GLuint,
}

impl State {
    pub fn new() -> State {
        State {
            current_program: 0,
            current_texture_2d: 0,
        }
    }
}
