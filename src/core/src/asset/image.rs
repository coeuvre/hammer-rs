use std::path::Path;
use std::ptr;
use std::ffi::CString;

use super::*;

use Error;

use util::stb_image::*;
use util::cstr_to_string;

pub struct Image {
    w: i32,
    h: i32,
    data: Vec<u8>,
}

impl Resource for Image {
    fn type_name() -> &'static str {
        "Image"
    }
}

unsafe impl Send for Image {}

impl<S: AsRef<Path>> Loadable<S> for Image {
    fn load(src: S) -> Result<Image, Error> {
        let path = src.as_ref();
        unsafe {
            let cstr = CString::new(&*path.as_os_str().to_string_lossy()).unwrap();
            let mut w = 0;
            let mut h = 0;
            let data = stbi_load(cstr.as_ptr(), &mut w, &mut h, ptr::null_mut(), 4);
            if data != ptr::null_mut() {
                let mut pixels = Vec::with_capacity((w * 4 * h) as usize);
                // Flip the pixels
                {
                    let data = ::std::slice::from_raw_parts(data, (w * 4 * h) as usize);
                    for row in data.chunks((w * 4) as usize).rev() {
                        pixels.extend(row);
                    }
                }
                stbi_image_free(data);
                Ok(Image {
                    w: w,
                    h: h,
                    data: pixels,
                })
            } else {
                Err(format!("Failed to load {}: {}", path.display(), cstr_to_string(stbi_failure_reason())).into())
            }
        }
    }
}

impl Image {
    pub fn size(&self) -> (i32, i32) {
        (self.w, self.h)
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
