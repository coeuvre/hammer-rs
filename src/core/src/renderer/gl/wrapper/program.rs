use std::str;
use std::ptr;
use std::ffi::CString;

use super::gl;
use super::gl::types::*;

use super::{Context, Shader};

use Error;

pub struct Program {
    context: Context,
    id: GLuint,
}

impl Program {
    pub fn link(context: &Context, vs: &Shader, fs: &Shader) -> Result<Program, Error> {
        if let &Shader::VertexShader(vs) = vs {
            if let &Shader::FragmentShader(fs) = fs {
                unsafe {
                    let program = gl::CreateProgram();
                    gl::AttachShader(program, vs);
                    gl::AttachShader(program, fs);
                    gl::LinkProgram(program);

                    let mut status = gl::FALSE as GLint;
                    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
                    if status == gl::TRUE as GLint {
                        Ok(Program {
                            context: context.clone(),
                            id: program,
                        })
                    } else {
                        let mut len: GLint = 0;
                        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                        let mut buf = Vec::with_capacity(len as usize);
                        buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                        gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);

                        gl::DeleteProgram(program);
                        Err(From::from(str::from_utf8(&buf).ok().expect("ProgramInfoLog not valid utf8")))
                    }
                }
            } else {
                Err(From::from("Second parameter is not a FragmentShader".to_string()))
            }
        } else {
            Err(From::from("First parameter is not a VertexShader".to_string()))
        }
    }

    pub fn compile_and_link(context: &Context, vsrc: &str, fsrc: &str) -> Result<Program, Error> {
        let vs = try!(Shader::compile(gl::VERTEX_SHADER, vsrc));
        let fs = try!(Shader::compile(gl::FRAGMENT_SHADER, fsrc));
        Program::link(context, &vs, &fs)
    }

    pub fn active(&self) {
        self.context.use_program(self.id);
    }

    pub fn set_uniform_1i(&mut self, uniform: &str, value: i32) {
        self.active();

        let loc = self.get_uniform_location(uniform);
        if loc != -1 {
            unsafe { gl::Uniform1i(loc, value); }
        } else {
            error!("Failed to set value for uniform `{}`", uniform);
        }
    }

    pub fn set_uniform_4f(&mut self, uniform: &str, v0: f32, v1: f32, v2: f32, v3: f32) {
        self.active();

        let loc = self.get_uniform_location(uniform);
        if loc != -1 {
            unsafe { gl::Uniform4f(loc, v0, v1, v2, v3); }
        } else {
            error!("Failed to set value for uniform `{}`", uniform);
        }
    }

    pub fn set_uniform_matrix3_fv(&mut self, uniform: &str, value: &[GLfloat]) {
        self.active();

        let loc = self.get_uniform_location(uniform);
        if loc != -1 {
            unsafe { gl::UniformMatrix3fv(loc, 1, gl::FALSE, value.as_ptr()); }
        } else {
            error!("Failed to set value for uniform `{}`", uniform);
        }
    }

    fn get_uniform_location(&self, uniform: &str) -> i32 {
        let cstr = CString::new(uniform).unwrap();
        unsafe { gl::GetUniformLocation(self.id, cstr.as_ptr()) }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id); }
    }
}
