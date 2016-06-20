use std::ptr;
use std::path::Path;
use std::os::raw::c_void;
use std::ffi::CString;

use super::gl;
use super::gl::types::*;

use super::Context;

use util::stb_image::*;
use util::cstr_to_string;

use Error;
use asset;

pub struct Texture {
    context: Context,
    id: GLuint,
    raw: asset::Texture
}

impl Texture {
    pub fn new(context: &Context, raw: &asset::Texture) -> Result<Texture, Error> {
        raw.access(|w, h, data| {
            unsafe {
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
                Ok(Texture {
                    context: context.clone(),
                    id: id,
                    raw: raw.clone(),
                })
            }
        }).unwrap_or(Err(From::from("Failed to access texture")))
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
