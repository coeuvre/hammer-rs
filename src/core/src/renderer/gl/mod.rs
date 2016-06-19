use std::path::Path;

use Error;

use window::Window;

use self::wrapper::*;
use super::Drawable;

use math::Trans;

mod wrapper;

pub struct Renderer {
    context: Context,
    quad: Quad,

    window_to_clip_trans: Trans,
}

impl Renderer {
    pub fn new(window: &Window) -> Result<Renderer, Error> {
        let context = Context::new(window);

        let quad = try!(Quad::new(&context));

        Ok(Renderer {
            context: context,
            quad: quad,

            window_to_clip_trans: Trans::identity(),
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

    pub fn ortho(&mut self, left: f32, right: f32, bottom: f32, top: f32) {
        // x -> (left, right)
        // x - left -> (0, right - left)
        // (x - left) / (right - left) * 2 - 1  -> (-1, 1)
        //
        // y -> (bottom, top)
        // y - bottom -> (0, top - bottom)
        // (y - bottom) / (top - bottom) * 2 - 1 -> (-1, 1)
        //
        let trans = Trans::offset(-left, - bottom);
        let trans = Trans::scale(2.0 / (right - left), 2.0 / (top - bottom)) * trans;
        self.window_to_clip_trans = Trans::offset(-1.0, -1.0) * trans;
    }

    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<Texture, Error> {
        Texture::load(&self.context, path)
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect {
            renderer: self,
            x: x,
            y: y,
            w: w,
            h: h,
        }
    }
}

pub struct Rect<'a> {
    renderer: &'a mut Renderer,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl<'a> Rect<'a> {
    pub fn texture<'b>(self, texture: &'b Texture) -> TexturedRect<'a, 'b> {
        TexturedRect {
            renderer: self.renderer,
            texture: texture,
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h,
        }
    }
}

pub struct TexturedRect<'a, 'b> {
    renderer: &'a mut Renderer,
    texture: &'b Texture,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl<'a, 'b> Drawable for TexturedRect<'a, 'b> {
    fn draw(&mut self) {
        self.renderer.quad.fill_with_texture(self.renderer.window_to_clip_trans, self.x, self.y, self.w, self.h, self.texture);
    }
}
