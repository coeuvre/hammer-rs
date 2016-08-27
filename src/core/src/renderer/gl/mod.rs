use std::collections::HashMap;

use Error;

use window::Window;

use self::wrapper::*;

use asset::*;
use math::*;

pub mod wrapper;

pub type TextureCache = HashMap<usize, Texture>;

pub struct Renderer {
    context: Context,
    quad: QuadProgram,

    textures: TextureCache,
}

impl Renderer {
    pub fn new(window: &Window) -> Result<Renderer, Error> {
        let context = Context::new(window);

        let quad = try!(QuadProgram::new(&context));

        Ok(Renderer {
            context: context,
            quad: quad,

            textures: TextureCache::new(),
        })
    }

    pub fn clear(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.context.clear_color(r, g, b, a);
        self.context.clear();
    }

    // pub fn resize(&mut self, w: i32, h: i32) {
    //     self.context.viewport(0, 0, w, h);
    // }

    pub fn present(&mut self) {
        self.context.swap_buffers();
    }

    pub fn fill_with_texture<T: AsTexture>(&mut self, trans: Transform, dst: Option<&Rect>, texture: &T) {
        if let Ok(texture) = texture.as_texture(&self.context, &mut self.textures) {
            self.quad.fill_with_texture(trans, dst.unwrap_or(&texture.option_dst), texture.texture, &texture.src);
        }
    }

    pub fn fill_with_color(&mut self, trans: Transform, dst: &Rect, r: f32, g: f32, b: f32, a: f32) {
        self.quad.fill_with_color(trans, dst, r, g, b, a);
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

pub struct TextureRef<'a> {
    texture: &'a Texture,
    src: Rect,
    option_dst: Rect,
}

pub trait AsTexture: Clone {
    fn as_texture<'r>(&self, context: &Context, textures: &'r mut TextureCache) -> Result<TextureRef<'r>, Error>;
}

impl<'a> AsTexture for ImageRef {
    fn as_texture<'r>(&self, context: &Context, textures: &'r mut TextureCache) -> Result<TextureRef<'r>, Error> {
        let id = self.read().id();
        if !textures.contains_key(&id) {
            match Texture::new(&context, &*self.read()) {
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
            src: Rect::with_min_size(vector(0.0, 0.0), size),
            option_dst: Rect::with_min_size(vector(0.0, 0.0), size),
        })
    }
}

impl<'a> AsTexture for Frame {
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
        let size = self.region().size();
        let anchor = self.anchor() % size;
        Ok(TextureRef {
            texture: texture,
            src: *self.region(),
            option_dst: Rect::with_min_size(-anchor, size),
        })
    }
}

impl<A: Asset + AsTexture> AsTexture for AssetRef<A> {
    fn as_texture<'r>(&self, context: &Context, textures: &'r mut TextureCache) -> Result<TextureRef<'r>, Error> {
        self.read().as_texture(context, textures)
    }
}
