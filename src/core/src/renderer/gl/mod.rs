extern crate stb_image;

use std::path::Path;

use Error;

use window::Window;

use self::wrapper::*;

mod wrapper;

pub struct Renderer {
    context: Context,

    quad: Quad,
}

impl Renderer {
    pub fn new(window: &Window) -> Result<Renderer, Error> {
        let context = Context::new(window);

        let quad = try!(Quad::new(&context));

        Ok(Renderer {
            context: context,
            quad: quad,
        })
    }

    pub fn clear(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.context.clear_color(r, g, b, a);
        self.context.clear();
    }

    pub fn resize(&mut self, w: i32, h: i32) {
        self.context.viewport(0, 0, w, h);
    }

    pub fn present(&mut self) {
        self.context.swap_buffers();
    }

/*
    fn prepare(&mut self) {
        unsafe {
            gl::Enable(gl::FRAMEBUFFER_SRGB);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            gl::BlendEquation(gl::FUNC_ADD);
        }
    }
*/

    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<Texture, Error> {
        use self::stb_image::image::*;

        match load_with_depth(path, 4, true) {
            LoadResult::Error(e) => {
                Err(From::from(e))
            }

            LoadResult::ImageU8(image) => {
                Texture::from_memory(&self.context, &image.data, image.width, image.height)
            }

            LoadResult::ImageF32(_) => unreachable!(),
        }
    }

    pub fn draw(&mut self, drawable: &Drawable) {
        drawable.draw(self);
    }
}

pub trait Drawable {
    fn draw(&self, renderer: &mut Renderer);
}

impl Drawable for Texture {
    fn draw(&self, renderer: &mut Renderer) {
        renderer.quad.fill_with_texture(self);
    }
}

pub struct Quad {
    context: Context,
    program: Program,
}

impl Quad {
    pub fn new(context: &Context) -> Result<Quad, Error> {
        const VERTEX_SHADER: &'static str = r#"
        #version 330 core

        layout (location = 0)
        in vec2 pos;

        layout (location = 1)
        in vec2 texcoord;

        out vec2 v_texcoord;

        void main() {
            gl_Position = vec4(pos.xy, 0.0, 1.0);
            v_texcoord = texcoord;
        }
        "#;

        const FRAGMENT_SHADER: &'static str = r#"
        #version 330 core

        uniform sampler2D u_texture0;

        in vec2 v_texcoord;

        out vec4 color;

        void main() {
            color = texture2D(u_texture0, v_texcoord);
        }
        "#;

        const VERTICES: [f32; 16] = [
            // Positions // Texture Coords
            0.0, 1.0,    0.0, 0.0,
            1.0, 1.0,    1.0, 0.0,
            0.0, 0.0,    0.0, 1.0,
            1.0, 0.0,    1.0, 1.0,
        ];

        let mut program = try!(Program::compile_and_link(context, VERTEX_SHADER, FRAGMENT_SHADER));

        program.set_uniform_1i("u_texture0", 0);

    /*
        glUniform1i(glGetUniformLocation(program.id, "u_texture0"), 0);

        glGenVertexArrays(1, &program.vao);
        glBindVertexArray(program.vao);

        glGenBuffers(1, &program.vbo);
        glBindBuffer(GL_ARRAY_BUFFER, program.vbo);
        glBufferData(GL_ARRAY_BUFFER, sizeof(QUAD_VERTICES), QUAD_VERTICES, GL_STATIC_DRAW);

        glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, 4 * sizeof(GLfloat), 0);
        glEnableVertexAttribArray(0);

        glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE, 4 * sizeof(GLfloat),
                              (GLvoid *)(2 * sizeof(GLfloat)));
        glEnableVertexAttribArray(1);

        glBindVertexArray(0);
    */
        Ok(Quad {
            context: context.clone(),
            program: program,
        })
    }

    pub fn fill_with_texture(&mut self, texture: &Texture) {
    }
}
