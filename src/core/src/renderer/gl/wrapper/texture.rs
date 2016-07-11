use std::os::raw::c_void;

use super::gl;
use super::gl::types::*;

use super::Context;

use Error;

use asset::image::Image;
use math::*;

pub struct Texture {
    context: Context,
    id: GLuint,
    size: Vector,
}

impl Texture {
    pub fn new(context: &Context, image: &Image) -> Result<Texture, Error> {
        let mut id = 0;

        let (w, h) = image.size();
        let data = image.data();
        let size = vector(w as f32, h as f32);

        unsafe {
            gl::GenTextures(1, &mut id);

            context.bind_texture_2d(id);

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::SRGB_ALPHA as i32, w as i32, h as i32, 0,
                           gl::RGBA, gl::UNSIGNED_BYTE, data.as_ptr() as *const c_void);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            context.bind_texture_2d(0);
        }

        Ok(Texture {
            context: context.clone(),
            id: id,
            size: size,
        })
    }

    pub fn active(&self, unit: u32) {
        self.context.active_texture(gl::TEXTURE0 + unit);
        self.context.bind_texture_2d(self.id);
    }

    pub fn size(&self) -> Vector {
        self.size
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id); }
    }
}
