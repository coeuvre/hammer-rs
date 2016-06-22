extern crate typemap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

pub mod prelude;
pub mod asset;
pub mod window;
pub mod renderer;
pub mod math;

mod util;

pub type Error = Box<std::error::Error + Send + Sync>;
