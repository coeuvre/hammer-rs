use std::str;
use std::ptr;
use std::ffi::CString;

use super::gl;
use super::gl::types::*;

use Error;

pub enum Shader {
    VertexShader(GLuint),
    FragmentShader(GLuint),
}

impl Shader {
    pub fn compile(ty: GLenum, src: &str) -> Result<Shader, Error> {
        unsafe {
            let shader = gl::CreateShader(ty);

            let cstr = CString::new(src).unwrap();
            gl::ShaderSource(shader, 1, &cstr.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
            if status == gl::TRUE as GLint {
                match ty {
                    gl::VERTEX_SHADER => {
                        Ok(Shader::VertexShader(shader))
                    }

                    gl::FRAGMENT_SHADER => {
                        Ok(Shader::FragmentShader(shader))
                    }

                    _ => unimplemented!()
                }
            } else {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);

                gl::DeleteShader(shader);
                Err(From::from(str::from_utf8(&buf).ok().expect("ShaderInfoLog not valid utf8")))
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        match *self {
            Shader::VertexShader(id) | Shader::FragmentShader(id) => {
                unsafe { gl::DeleteShader(id); }
            }
        }
    }
}
