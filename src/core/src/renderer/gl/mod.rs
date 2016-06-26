use std::collections::HashMap;

use Error;

use window::Window;

use self::wrapper::*;
use super::Drawable;

use asset;
use math::Trans;

pub mod wrapper;

pub type TextureCache = HashMap<u64, Texture>;

pub struct Renderer {
    context: Context,
    quad: Quad,

    window_to_clip_trans: Trans,

    textures: TextureCache,
}

impl Renderer {
    pub fn new(window: &Window) -> Result<Renderer, Error> {
        let context = Context::new(window);

        let quad = try!(Quad::new(&context));

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

pub trait AsTexture {
    fn as_texture<'r>(&self, context: &Context, textures: &'r mut TextureCache) -> Result<&'r Texture, Error>;
}

impl<'a> AsTexture for asset::image::Image {
    fn as_texture<'r>(&self, context: &Context, textures: &'r mut TextureCache) -> Result<&'r Texture, Error> {
        let id = self.id();
        if !textures.contains_key(&id) {
            match Texture::new(&context, self) {
                Ok(texture) => {
                    textures.insert(id, texture);
                }

                Err(e) => return Err(e),
            }
        }

        Ok(textures.get(&id).unwrap())
    }
}

impl AsTexture for asset::Handle<asset::image::Image> {
    fn as_texture<'r>(&self, context: &Context, textures: &'r mut TextureCache) -> Result<&'r Texture, Error> {
        match self.read() {
            Some(image) => image.as_texture(context, textures),
            None => Err("Failed to read image".into()),
        }
    }
}

impl<'a> AsTexture for asset::AssetLockReadGuard<'a, asset::image::Image> {
    fn as_texture<'r>(&self, context: &Context, textures: &'r mut TextureCache) -> Result<&'r Texture, Error> {
        ((&(**self) as &asset::image::Image) as &AsTexture).as_texture(context, textures)
    }
}

impl<'a> Rect<'a> {
    pub fn texture<'b, T: AsTexture + 'b>(self, texture: &'b T) -> TexturedRect<'a, 'b, T> {
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

pub struct TexturedRect<'a, 'b, T: 'b> {
    renderer: &'a mut Renderer,
    texture: &'b T,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl<'a, 'b, T: AsTexture + 'b> Drawable for TexturedRect<'a, 'b, T> {
    fn draw(&mut self) {
        if let Ok(texture) = self.texture.as_texture(&self.renderer.context, &mut self.renderer.textures) {
            self.renderer.quad.fill_with_texture(self.renderer.window_to_clip_trans, self.x, self.y, self.w, self.h, texture);
        }
    }
}
