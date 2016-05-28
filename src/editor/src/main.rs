#![allow(unused_imports)]
#![feature(const_fn)]

#[macro_use]
extern crate lazy_static;

extern crate gl;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate winapi;
extern crate gdi32;
extern crate kernel32;
extern crate opengl32;
extern crate user32;

use std::mem;
use std::ffi::{CString, CStr};
use std::thread;

use winapi::basetsd::*;
use winapi::minwindef::*;
use winapi::windef::*;
use winapi::wingdi::*;
use winapi::winnt::*;
use winapi::winuser::*;
use gdi32::*;
use kernel32::*;
use opengl32::*;
use user32::*;

use gl::types::*;

use window::*;

#[macro_use]
mod window;

pub type Error = Box<std::error::Error + Send + Sync>;

fn main() {
    env_logger::init().unwrap();

    let thread_handle = thread::spawn(move || {
        let mut window = WindowBuilder::new().title("Window 2").pos(0, 600).build().unwrap();
        window.show();

        let mut renderer = Renderer::new();

        let mut context = renderer.active(&window).unwrap();

        // Game like loop
        'event_loop: loop {
            for event in window.poll_events() {
                match event {
                    Event::Close => break 'event_loop,
                    Event::Resize { w, h, .. } => {
                        context.resize(w, h);
                        unsafe {
                            gl::ClearColor(1.0, 0.0, 0.0, 1.0);
                            gl::Clear(gl::COLOR_BUFFER_BIT);
                        }
                        context.present();
                    }
                }
            }
            thread::yield_now();
        }

        window.close();
    });

    let mut window = WindowBuilder::new().title("Window 1").build().unwrap();
    window.show();

    let mut renderer = Renderer::new();

    let mut context = renderer.active(&window).unwrap();

    // GUI like loop
    for event in window.wait_events() {
        match event {
            Event::Close => break,
            Event::Resize { w, h, .. } => {
                context.resize(w, h);
                unsafe {
                    gl::ClearColor(1.0, 1.0, 1.0, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }
                context.present();
            }
        }
    }

    window.close();

    thread_handle.join().unwrap();
}
