use std::collections::HashMap;

use Error;

use window::Window;

use self::wrapper::*;
use super::Drawable;

use asset;
use math::Trans;

pub mod wrapper;

pub struct Renderer {
    context: Context,
    quad: Quad,

    window_to_clip_trans: Trans,

    textures: HashMap<asset::AssetId, Texture>,
}

impl Renderer {
    pub fn new(window: &Window) -> Result<Renderer, Error> {
        let context = Context::new(window);

        let quad = try!(Quad::new(&context));

        Ok(Renderer {
            context: context,
            quad: quad,

            window_to_clip_trans: Trans::identity(),
            textures: HashMap::new(),
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
    fn id(&self) -> &asset::AssetId;
    fn as_texture(&self, renderer: &mut Renderer) -> Result<Texture, Error>;
}

impl AsTexture for asset::Asset<asset::image::Image> {
    fn id(&self) -> &asset::AssetId {
        self.id()
    }

    fn as_texture(&self, renderer: &mut Renderer) -> Result<Texture, Error> {
        try!(self.access(|image| {
            Texture::new(&renderer.context, &image)
        }))
    }
}

impl<'a> AsTexture for asset::AssetRef<'a, asset::image::Image> {
    fn id(&self) -> &asset::AssetId {
        self.id()
    }

    fn as_texture(&self, renderer: &mut Renderer) -> Result<Texture, Error> {
        Texture::new(&renderer.context, self)
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
        if !self.renderer.textures.contains_key(self.texture.id()) {
            match self.texture.as_texture(self.renderer) {
                Ok(texture) => {
                    self.renderer.textures.insert(self.texture.id().clone(), texture);
                    info!("Uploaded Image({}) to GPU.", self.texture.id());
                }

                Err(e) => {
                    warn!("{}", e)
                }
            }
        }

        if let Some(texture) = self.renderer.textures.get(self.texture.id()) {
            self.renderer.quad.fill_with_texture(self.renderer.window_to_clip_trans, self.x, self.y, self.w, self.h, texture);
        }
    }
}
