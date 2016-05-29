#![allow(unused_imports)]
#![feature(const_fn)]

#[macro_use]
extern crate lazy_static;

extern crate gl;

#[macro_use]
extern crate log;
extern crate env_logger;

use std::thread;

use window::*;

#[macro_use]
mod window;

pub type Error = Box<std::error::Error + Send + Sync>;

fn main() {
    env_logger::init().unwrap();

    let thread_handle = thread::spawn(move || {
        let mut window = WindowBuilder::new().title("Window 2").pos(0, 600).build().unwrap();
        window.show();

        let mut renderer = Renderer::new(&window);

        // Game like loop
        'event_loop: loop {
            for event in window.poll_events() {
                match event {
                    Event::Close => break 'event_loop,
                    Event::Resize { w, h, .. } => {
                        renderer.resize(w, h);
                        unsafe {
                            gl::ClearColor(1.0, 0.0, 0.0, 1.0);
                            gl::Clear(gl::COLOR_BUFFER_BIT);
                        }
                        renderer.present();
                    }
                }
            }
            thread::yield_now();
        }

        window.close();
    });

    let mut window = WindowBuilder::new().title("Window 1").size(640, 480).build().unwrap();
    window.show();

    let mut renderer = Renderer::new(&window);

    // GUI like loop
    for event in window.wait_events() {
        match event {
            Event::Close => break,
            Event::Resize { w, h, .. } => {
                renderer.resize(w, h);
                unsafe {
                    gl::ClearColor(1.0, 1.0, 1.0, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }
                renderer.present();
            }
        }
    }

    window.close();

    thread_handle.join().unwrap();
}
