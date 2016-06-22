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
    data: *mut u8,
}

impl Resource for Image {
    fn type_name() -> &'static str {
        "Image"
    }
}
unsafe impl Send for Image {}

impl Loadable for Image {
    fn load<P: AsRef<Path>>(path: P) -> Result<Image, Error> {
        unsafe {
            let cstr = CString::new(&*path.as_ref().as_os_str().to_string_lossy()).unwrap();
            let mut w = 0;
            let mut h = 0;
            let data = stbi_load(cstr.as_ptr(), &mut w, &mut h, ptr::null_mut(), 4);
            if data != ptr::null_mut() {
                Ok(Image {
                    w: w,
                    h: h,
                    data: data,
                })
            } else {
                Err(format!("Failed to load {}: {}", path.as_ref().display(), cstr_to_string(stbi_failure_reason())).into())
            }
        }
    }
}

impl Image {
    pub fn size(&self) -> (i32, i32) {
        (self.w, self.h)
    }

    pub unsafe fn data(&self) -> *mut u8 {
        self.data
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            stbi_image_free(self.data);
        }
    }
}
