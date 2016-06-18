#[macro_use]
extern crate log;

pub mod window;
pub mod renderer;

mod util;

pub type Error = Box<std::error::Error + Send + Sync>;
