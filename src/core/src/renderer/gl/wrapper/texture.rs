use std::os::raw::c_void;

use super::gl;
use super::gl::types::*;

use super::Context;

use Error;

pub struct Texture {
    context: Context,
    id: GLuint,
}

impl Texture {
    pub fn from_memory(context: &Context, bytes: &[u8], width: usize, height: usize) -> Result<Texture, Error> {
        assert!(bytes.len() == width * 4 * height);

        unsafe {
            let mut id = 0;

            gl::GenTextures(1, &mut id);

            context.bind_texture_2d(id);

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::SRGB_ALPHA as i32, width as i32, height as i32, 0,
                           gl::RGBA, gl::UNSIGNED_BYTE, bytes.as_ptr() as *const c_void);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            context.bind_texture_2d(0);

            Ok(Texture {
                context: context.clone(),
                id: id,
            })
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id); }
    }
}
