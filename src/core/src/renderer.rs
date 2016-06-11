use window::{Window, RenderContext};

use gl;

pub struct Renderer {
    context: RenderContext,
}

impl Renderer {
    pub fn new(window: &Window) -> Renderer {
        Renderer {
            context: window.create_render_context(),
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

}
