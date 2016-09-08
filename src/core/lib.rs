#![feature(step_trait)]

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate typemap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

pub mod prelude;

pub mod asset;
// pub mod renderer;
pub mod math;
pub mod util;
pub mod input;

pub type Error = Box<std::error::Error + Send + Sync>;
