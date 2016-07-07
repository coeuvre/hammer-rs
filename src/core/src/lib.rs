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
use ecs::system::*;
use math::Transform;

use std::collections::HashSet;
use std::time::{Duration, Instant};

pub fn run(scene: Scene, mut pre_render_systems: Vec<Box<System>>, mut render_systems: Vec<Box<System>>) {
    let mut started_scene: HashSet<String> = HashSet::new();

    scene::push(scene);

    let mut window = window::WindowBuilder::new().size(800, 600).build().unwrap();
    window.show();

    renderer::set_target(&window);

    pre_render_systems.push(Box::new(BehaviourSystem {}));
    pre_render_systems.push(Box::new(CameraSystem::new()));

    render_systems.push(Box::new(SpriteSystem {}));

    let frame_time = Duration::from_millis(16);

    'game_loop: loop {
        let frame_start = Instant::now();

        input::keyboard::update();

        for event in window.poll_events() {
            match event {
                window::Event::Close => break 'game_loop,
                window::Event::KeyDown(key) => input::keyboard::set_down(key),
                window::Event::KeyUp(key) => input::keyboard::set_up(key),
                _ => {}
            }
        }

        match scene::top() {
            Some(scene) => {
                if !started_scene.contains(scene.read().id()) {
                    start_entity(&mut pre_render_systems, scene.read().root());
                    start_entity(&mut render_systems, scene.read().root());
                    started_scene.insert(scene.read().id().to_string());
                }

                frame_begin(&mut pre_render_systems);
                frame_begin(&mut render_systems);

                update_entity(&mut pre_render_systems, scene.read().root());

                renderer::with_camera(|camera| {
                    let (r, g, b, a) = camera.background();
                    renderer::clear(r, g, b, a);

                    let region = camera.region();
                    let projection = Transform::ortho(region.left(), region.right(), region.bottom(), region.top()) * camera.transform();
                    renderer::set_projection(projection);

                    update_entity(&mut render_systems, scene.read().root());

                    renderer::present();
                });

                frame_end(&mut pre_render_systems);
                frame_end(&mut render_systems);
            }

            None => break 'game_loop,
        }

        // {
        //     let elapsed = frame_start.elapsed();
        //     info!("Frame time {} ms", ((elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0) * 1000.0) as u32);
        // }

        if frame_time > frame_start.elapsed() {
            std::thread::sleep(frame_time - frame_start.elapsed());
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

fn frame_begin(systems: &mut Vec<Box<System>>) {
    for system in systems.iter_mut() {
        system.frame_begin();
    }
}

fn frame_end(systems: &mut Vec<Box<System>>) {
    for system in systems.iter_mut() {
        system.frame_end();
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
