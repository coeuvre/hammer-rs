use super::gl;
use super::gl::types::*;

use Error;

pub struct Texture {
    id: GLuint,
}

impl Texture {
    pub fn load() -> Result<Texture, Error> {
        unimplemented!();
    }
}
