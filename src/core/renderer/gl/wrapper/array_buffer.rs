use std::mem;
use std::ptr;

use std::os::raw::c_void;

use super::gl;
use super::gl::types::*;

use super::{Context, BufferUsage};

pub struct ArrayBuffer {
    context: Context,
    id: GLuint,
}

impl ArrayBuffer {
    pub fn new(context: &Context) -> ArrayBuffer {
        unsafe {
            let mut id = 0;
            gl::GenBuffers(1, &mut id);
            ArrayBuffer {
                context: context.clone(),
                id: id,
            }
        }
    }

    pub fn bind(&self) {
        self.context.bind_array_buffer(self.id);
    }

    pub fn buffer_data<T: Sized>(&mut self, data: Option<&[T]>, usage: BufferUsage) {
        self.bind();

        if let Some(data) = data {
            unsafe { gl::BufferData(gl::ARRAY_BUFFER, (mem::size_of::<T>() * data.len()) as GLsizeiptr, data.as_ptr() as *const c_void, usage.to_gl()); }
        } else {
            unsafe { gl::BufferData(gl::ARRAY_BUFFER, 0, ptr::null(), usage.to_gl()); }
        }
    }
}
