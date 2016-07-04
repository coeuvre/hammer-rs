#![feature(zero_one)]

extern crate typemap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

pub mod prelude;

pub mod asset;
pub mod scene;
pub mod window;
pub mod renderer;
pub mod math;
pub mod util;
pub mod input;
pub mod ecs;

pub type Error = Box<std::error::Error + Send + Sync>;

use scene::*;
use ecs::*;

pub fn run(mut scene: Scene) {
    let mut window = window::WindowBuilder::new().size(800, 600).build().unwrap();
    window.show();

    let renderer = renderer::Renderer::new(&window).unwrap();
    let mut render_system = RenderSystem::new(renderer);

    let mut behaviour_system = BehaviourSystem {};

    behaviour_system.start(&mut scene);

    'game_loop: loop {
        for event in window.poll_events() {
            match event {
                window::Event::Close => break 'game_loop,
                _ => {}
            }
        }

        behaviour_system.update(&mut scene);

        render_system.process(&mut scene);
    }
}
