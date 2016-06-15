use std::str;
use std::ptr;
use std::ffi::CString;

use gl;
use gl::types::*;

use Error;
use window::{Window, GlContext};

pub struct Renderer {
    context: GlContext,
}

impl Renderer {
    pub fn new(window: &Window) -> Renderer {
        Renderer {
            context: window.create_gl_context(),
        }
    }

    pub fn resize(&mut self, w: i32, h: i32) {
        self.context.make_current();

        unsafe {
            //self.buffer.resize(w, h);
            gl::Viewport(0, 0, w, h);
        }
    }

    pub fn clear(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.context.make_current();

        unsafe {
            gl::ClearColor(r, g, b, a);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn present(&mut self) {
        self.context.make_current();
        self.context.swap_buffers();
    }
}

pub struct Texture {
    id: GLuint,
}

impl Texture {
    pub fn load() -> Result<Texture, Error> {
        unimplemented!();
    }
}

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
