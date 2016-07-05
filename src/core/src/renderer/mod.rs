use std::cell::RefCell;

mod gl;

use self::gl::Renderer;

use math::*;
use window::Window;

pub trait Drawable {
    fn draw(&mut self);
}

pub struct Quad {
    rect: Rect,
}

impl Quad {
    pub fn texture<'b, T: gl::AsTexture + 'b>(self, texture: &'b T) -> TexturedQuad<'b, T> {
        TexturedQuad {
            texture: texture,
            dst: self.rect,
        }
    }
}

pub struct TexturedQuad<'b, T: 'b> {
    texture: &'b T,
    dst: Rect,
}

impl<'b, T: gl::AsTexture + 'b> Drawable for TexturedQuad<'b, T> {
    fn draw(&mut self) {
        CONTEXT.with(|context| {
            if let Some(ref mut renderer) = *context.renderer.borrow_mut() {
                renderer.fill_with_texture(&self.dst, self.texture);
            }
        });
    }
}

struct Context {
    renderer: RefCell<Option<Renderer>>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            renderer: RefCell::new(None),
        }
    }

    pub fn set_target(&self, window: &Window) {
        *self.renderer.borrow_mut() = Renderer::new(window).ok();
    }

    pub fn set_transform(&self, trans: Transform) {
        if let Some(ref mut renderer) = *self.renderer.borrow_mut() {
            renderer.set_transform(trans);
        }
    }

    pub fn set_projection(&self, trans: Transform) {
        if let Some(ref mut renderer) = *self.renderer.borrow_mut() {
            renderer.set_projection(trans);
        }
    }

    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        if let Some(ref mut renderer) = *self.renderer.borrow_mut() {
            renderer.clear(r, g, b, a);
        }
    }

    pub fn present(&self) {
        if let Some(ref mut renderer) = *self.renderer.borrow_mut() {
            renderer.present();
        }
    }
}

thread_local!(static CONTEXT: Context = Context::new());

pub fn set_target(window: &Window) {
    CONTEXT.with(|context| context.set_target(window))
}

pub fn clear(r: f32, g: f32, b: f32, a: f32) {
    CONTEXT.with(|context| context.clear(r, g, b, a))
}

pub fn present() {
    CONTEXT.with(|context| context.present())
}

pub fn set_transform(trans: Transform) {
    CONTEXT.with(|context| context.set_transform(trans))
}

pub fn set_projection(trans: Transform) {
    CONTEXT.with(|context| context.set_projection(trans))
}

pub fn rect(rect: Rect) -> Quad {
    Quad {
        rect: rect,
    }
}
