use std::str;
use std::ptr;
use std::ffi::CString;

use super::gl;
use super::gl::types::*;

use Error;

enum Shader {
    VertexShader(GLuint),
    FragmentShader(GLuint),
}

impl Shader {
    unsafe fn compile(ty: GLenum, src: &str) -> Result<Shader, Error> {
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

impl Drop for Shader {
    fn drop(&mut self) {
        match *self {
            Shader::VertexShader(id) | Shader::FragmentShader(id) => {
                unsafe { gl::DeleteShader(id); }
            }
        }
    }
}

struct Program {
    id: GLuint,
}

impl Program {
    unsafe fn link(vs: &Shader, fs: &Shader) -> Result<Program, Error> {
        if let &Shader::VertexShader(vs) = vs {
            if let &Shader::VertexShader(fs) = fs {
                let program = gl::CreateProgram();
                gl::AttachShader(program, vs);
                gl::AttachShader(program, fs);
                gl::LinkProgram(program);

                let mut status = gl::FALSE as GLint;
                gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
                if status == gl::TRUE as GLint {
                    Ok(Program {
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
            } else {
                Err(From::from("Second parameter is not a FragmentShader".to_string()))
            }
        } else {
            Err(From::from("First parameter is not a VertexShader".to_string()))
        }
    }

    unsafe fn compile_and_link(vsrc: &str, fsrc: &str) -> Result<Program, Error> {
        let vs = try!(Shader::compile(gl::VERTEX_SHADER, vsrc));
        let fs = try!(Shader::compile(gl::FRAGMENT_SHADER, fsrc));
        Program::link(&vs, &fs)
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id); }
    }
}
