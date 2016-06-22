use std::os::raw::c_void;

use super::gl;
use super::gl::types::*;

use super::Context;

use Error;
use asset;

pub struct Texture {
    context: Context,
    id: GLuint,
    _raw: asset::Asset<asset::image::Image>,
}

impl Texture {
    pub fn new(context: &Context, image: &asset::Asset<asset::image::Image>) -> Result<Texture, Error> {
        if let Some(raw) = image.borrow() {
            unsafe {
                let (w, h) = raw.size();
                let data = raw.data();

                let mut id = 0;
                gl::GenTextures(1, &mut id);

                context.bind_texture_2d(id);

                gl::TexImage2D(gl::TEXTURE_2D, 0, gl::SRGB_ALPHA as i32, w as i32, h as i32, 0,
                               gl::RGBA, gl::UNSIGNED_BYTE, data as *const c_void);

                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

                context.bind_texture_2d(0);

                info!("Uploaded texture `{}` into GPU.", image.id());

                Ok(Texture {
                    context: context.clone(),
                    id: id,
                    _raw: image.clone(),
                })
            }
        } else {
            Err(From::from(format!("Failed to access texture `{}`", image.id())))
        }
    }

    pub fn active(&self, unit: u32) {
        self.context.active_texture(gl::TEXTURE0 + unit);
        self.context.bind_texture_2d(self.id);
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id); }
    }
}
