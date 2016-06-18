use super::gl;
use super::gl::types::*;

use super::{Context, ArrayBuffer};

pub struct VertexArray {
    context: Context,
    id: GLuint,
}

impl VertexArray {
    pub fn new(context: &Context) -> VertexArray {
        unsafe {
            let mut id = 0;
            gl::GenVertexArrays(1, &mut id);
            VertexArray {
                context: context.clone(),
                id: id,
            }
        }
    }

    pub fn bind(&self) {
        self.context.bind_vertex_array(self.id);
    }

    pub fn bind_array_buffer(&mut self, array_buffer: &ArrayBuffer) {
        self.bind();
        array_buffer.bind();
        self.context.bind_vertex_array(0);
    }
}
