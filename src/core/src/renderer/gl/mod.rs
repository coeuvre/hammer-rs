use std::collections::HashMap;

use Error;

use window::Window;

use self::wrapper::*;
use super::Drawable;

use asset::*;
use math::*;

pub mod wrapper;

pub type TextureCache = HashMap<u64, Texture>;

pub struct Renderer {
    context: Context,
    quad: QuadProgram,

    window_to_clip_trans: Trans,

    textures: TextureCache,
}

impl Renderer {
    pub fn new(window: &Window) -> Result<Renderer, Error> {
        let context = Context::new(window);

        let quad = try!(QuadProgram::new(&context));

        Ok(Renderer {
            context: context,
            quad: quad,

            window_to_clip_trans: Trans::identity(),
            textures: TextureCache::new(),
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

    pub fn rect(&mut self, rect: Rect) -> Quad {
        Quad {
            renderer: self,
            rect: rect,
        }
    }
}

pub struct Quad<'a> {
    renderer: &'a mut Renderer,
    rect: Rect,
}

pub struct TextureRef<'a> {
    texture: &'a Texture,
    src: Rect,
}

pub trait AsTexture {
    fn as_texture<'r>(&self, context: &Context, textures: &'r mut TextureCache) -> Result<TextureRef<'r>, Error>;
}

impl<'a> AsTexture for Image {
    fn as_texture<'r>(&self, context: &Context, textures: &'r mut TextureCache) -> Result<TextureRef<'r>, Error> {
        let id = self.id();
        if !textures.contains_key(&id) {
            match Texture::new(&context, self) {
                Ok(texture) => {
                    textures.insert(id, texture);
                }

                Err(e) => return Err(e),
            }
        }

        let texture = textures.get(&id).unwrap();
        let size = texture.size();
        Ok(TextureRef {
            texture: texture,
            src: Rect::with_min_size(vec2(0.0, 0.0), size)
        })
    }
}

impl<'a> AsTexture for Sprite {
    fn as_texture<'r>(&self, context: &Context, textures: &'r mut TextureCache) -> Result<TextureRef<'r>, Error> {
        let image = self.image().read();
        let id = image.id();
        if !textures.contains_key(&id) {
            match Texture::new(&context, &*image) {
                Ok(texture) => {
                    textures.insert(id, texture);
                }

                Err(e) => return Err(e),
            }
        }

        let texture = textures.get(&id).unwrap();
        Ok(TextureRef {
            texture: texture,
            src: *self.region(),
        })
    }
}

impl<A: Asset + AsTexture> AsTexture for AssetRef<A> {
    fn as_texture<'r>(&self, context: &Context, textures: &'r mut TextureCache) -> Result<TextureRef<'r>, Error> {
        self.read().as_texture(context, textures)
    }
}

impl<A: Asset> AsTexture for Slot<A> where AssetRef<A>: AsTexture {
    fn as_texture<'r>(&self, context: &Context, textures: &'r mut TextureCache) -> Result<TextureRef<'r>, Error> {
        match self.get() {
            Some(asset) => asset.as_texture(context, textures),
            None => Err(format!("Failed to read {}", self).into()),
        }
    }
}

impl<'a> Quad<'a> {
    pub fn texture<'b, T: AsTexture + 'b>(self, texture: &'b T) -> TexturedQuad<'a, 'b, T> {
        TexturedQuad {
            renderer: self.renderer,
            texture: texture,
            dst: self.rect,
        }
    }
}

pub struct TexturedQuad<'a, 'b, T: 'b> {
    renderer: &'a mut Renderer,
    texture: &'b T,
    dst: Rect,
}

impl<'a, 'b, T: AsTexture + 'b> Drawable for TexturedQuad<'a, 'b, T> {
    fn draw(&mut self) {
        if let Ok(texture) = self.texture.as_texture(&self.renderer.context, &mut self.renderer.textures) {
            self.renderer.quad.fill_with_texture(self.renderer.window_to_clip_trans, &self.dst, texture.texture, &texture.src);
        }
    }
}
