extern crate gl;

#[macro_use]
extern crate log;

#[macro_use]
pub mod window;
pub mod renderer;

pub type Error = Box<std::error::Error + Send + Sync>;
