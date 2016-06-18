extern crate gl;

use std::sync::*;

use Error;
use window::{Window, GlContext};

use self::gl::types::*;

mod shader;
mod texture;

pub struct Renderer {
    context: GlContext,
}

static OPENGL_FUNCTION_INIT: Once = ONCE_INIT;

impl Renderer {
    pub fn new(window: &Window) -> Renderer {
        let mut context = window.create_gl_context();

        // TODO: Make sure that initialize OpenGL function once is enough.
        OPENGL_FUNCTION_INIT.call_once(|| {
            context.make_current();

            gl::load_with(|symbol| { context.load_function(symbol) });
        });

        Renderer {
            context: context,
        }
    }

    pub fn resize(&mut self, w: i32, h: i32) {
        self.context.make_current();

        unsafe {
            //self.buffer.resize(w, h);
            gl::Viewport(0, 0, w, h);
        }
    }

    pub fn clear(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.context.make_current();

        unsafe {
            gl::ClearColor(r, g, b, a);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn present(&mut self) {
        self.context.make_current();
        self.context.swap_buffers();
    }

/*
    fn prepare(&mut self) {
        unsafe {
            gl::Enable(gl::FRAMEBUFFER_SRGB);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            gl::BlendEquation(gl::FUNC_ADD);
        }
    }
*/
}
