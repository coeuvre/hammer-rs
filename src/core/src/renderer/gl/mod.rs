use std::path::Path;

use Error;

use window::Window;

use self::wrapper::*;

mod wrapper;

pub struct Renderer {
    context: Context,

    quad: Quad,
}

impl Renderer {
    pub fn new(window: &Window) -> Result<Renderer, Error> {
        let context = Context::new(window);

        let quad = try!(Quad::new(&context));

        Ok(Renderer {
            context: context,
            quad: quad,
        })
    }

    pub fn clear(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.context.clear_color(r, g, b, a);
        self.context.clear();
    }

    pub fn resize(&mut self, w: i32, h: i32) {
        self.context.viewport(0, 0, w, h);
    }

    pub fn present(&mut self) {
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

    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<Texture, Error> {
        Texture::load(&self.context, path)
    }

    pub fn draw(&mut self, drawable: &Drawable) {
        drawable.draw(self);
    }
}

pub trait Drawable {
    fn draw(&self, renderer: &mut Renderer);
}

impl Drawable for Texture {
    fn draw(&self, renderer: &mut Renderer) {
        renderer.quad.fill_with_texture(self);
    }
}
