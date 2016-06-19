pub type Renderer = gl::Renderer;

mod gl;

pub trait Drawable {
    fn draw(&mut self);
}
