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

pub struct Texture {
    context: Context,
    id: GLuint,
    w: i32,
    h: i32,
}

impl Texture {
    pub fn load<P: AsRef<Path>>(context: &Context, path: P) -> Result<Texture, Error> {
        let cstr = try!(CString::new(&*path.as_ref().as_os_str().to_string_lossy()));

        unsafe {
            let mut w = 0;
            let mut h = 0;
            let data = stbi_load(cstr.as_ptr(), &mut w, &mut h, ptr::null_mut(), 4);
            if data != ptr::null_mut() {
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

                stbi_image_free(data);

                info!("Loaded texture {} ({}x{})", path.as_ref().display(), w, h);

                Ok(Texture {
                    context: context.clone(),
                    id: id,
                    w: w,
                    h: h,
                })
            } else {
                Err(From::from(format!("Failed to load texture {}: {}", path.as_ref().display(), cstr_to_string(stbi_failure_reason()))))
            }
        }
    }

    pub fn active(&self, unit: u32) {
        self.context.active_texture(gl::TEXTURE0 + unit);
        self.context.bind_texture_2d(self.id);
    }

    pub fn size(&self) -> (i32, i32) {
        (self.w, self.h)
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id); }
    }
}
