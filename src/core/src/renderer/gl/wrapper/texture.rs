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
}

impl Texture {
    pub fn load<P: AsRef<Path>>(context: &Context, path: P) -> Result<Texture, Error> {
        let cstr = match path.as_ref().as_os_str().to_str() {
            Some(s) => match CString::new(s.as_bytes()) {
                Ok(s) => s.as_ptr(),
                Err(_) => return Err(From::from("path contains null character".to_string()))
            },
            None => return Err(From::from("path is not valid utf8".to_string())),
        };

        unsafe {
            let mut w = 0;
            let mut h = 0;
            let data = stbi_load(cstr, &mut w, &mut h, ptr::null_mut(), 4);
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

                info!("Loaded texture {} ({}x{})", cstr_to_string(cstr), w, h);

                Ok(Texture {
                    context: context.clone(),
                    id: id,
                })
            } else {
                Err(From::from(format!("Failed to load texture {}: {}", cstr_to_string(cstr), cstr_to_string(stbi_failure_reason()))))
            }
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id); }
    }
}
