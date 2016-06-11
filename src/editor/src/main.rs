#![allow(unused_imports)]
#![feature(const_fn)]

extern crate hammer_core;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;
extern crate env_logger;

use std::thread;

use hammer_core::window::*;
use hammer_core::renderer::*;

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
                        // renderer.resize(w, h);
                    }
                }
            }

            renderer.clear(1.0, 0.0, 0.0, 1.0);
            renderer.present();
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
                renderer.clear(1.0, 1.0, 1.0, 1.0);
                renderer.present();
            }
        }
    }

    window.close();

    thread_handle.join().unwrap();
}
