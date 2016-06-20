pub type Renderer = gl::Renderer;
pub type Texture = gl::wrapper::Texture;

mod gl;

pub trait Drawable {
    fn draw(&mut self);
}
