#[macro_use]
extern crate log;

pub mod event;

pub mod windows;
pub use windows::*;

// TODO: Add concrete error type
pub type Error = ();
