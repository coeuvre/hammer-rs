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
use math::Transform;
use input::keyboard::Key;

use std::collections::HashSet;

pub fn run(scene: Scene, mut systems: Vec<Box<System>>) {
    let mut started_scene: HashSet<String> = HashSet::new();

    scene::push(scene);

    let mut window = window::WindowBuilder::new().size(800, 600).build().unwrap();
    window.show();

    renderer::set_target(&window);

    systems.push(Box::new(RenderSystem {}));
    systems.push(Box::new(BehaviourSystem {}));

    let view_w = 640.0;
    let view_h = 480.0;
    renderer::set_projection(Transform::ortho(0.0, view_w, 0.0, view_h));

    'game_loop: loop {
        input::keyboard::update();

        for event in window.poll_events() {
            match event {
                window::Event::Close => break 'game_loop,
                window::Event::KeyDown(key) => input::keyboard::set_down(key),
                window::Event::KeyUp(key) => input::keyboard::set_up(key),
                _ => {}
            }
        }

        if input::keyboard::down(Key::Escape) {
            break 'game_loop;
        }

        renderer::clear(0.0, 0.0, 0.0, 1.0);

        match scene::top() {
            Some(scene) => {
                if !started_scene.contains(scene.read().id()) {
                    start_entity(&mut systems, scene.read().root());
                    started_scene.insert(scene.read().id().to_string());
                }

                update_entity(&mut systems, scene.read().root());

                renderer::present();
            }

            None => break 'game_loop,
        }
    }
}

fn start_entity(systems: &mut Vec<Box<System>>, entity: Entity) {
    if let Some(ref entity) = entity.get_ref() {
        for system in systems.iter_mut() {
            system.start(entity);
        }

        for child in entity.children() {
            start_entity(systems, child);
        }
    }
}

fn update_entity(systems: &mut Vec<Box<System>>, entity: Entity) {
    if let Some(ref entity) = entity.get_ref() {
        for system in systems.iter_mut() {
            system.update(entity);
        }

        for child in entity.children() {
            update_entity(systems, child);
        }
    }
}
