use std::ffi::CStr;
use std::os::raw::c_char;

pub mod stb_image;

pub fn cstr_to_string(ptr: *const c_char) -> String {
    unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}
