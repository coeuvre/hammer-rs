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

    pre_render_systems.push(Box::new(AnimationSystem {}));
    pre_render_systems.push(Box::new(BehaviourSystem {}));
    pre_render_systems.push(Box::new(CameraSystem::new()));
    pre_render_systems.push(Box::new(CollisionSystem::new()));

    render_systems.push(Box::new(SpriteSystem::new()));

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

        WORLD.with(|world| world.update(input::delta()));

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
                post_update_entity(&mut pre_render_systems, scene.read().root());

                renderer::with_camera(|camera| {
                    let (r, g, b, a) = camera.background();
                    renderer::clear(r, g, b, a);

                    let projection = Transform::ortho(*camera.region()) * camera.transform();
                    renderer::set_projection(projection);

                    update_entity(&mut render_systems, scene.read().root());
                    post_update_entity(&mut render_systems, scene.read().root());

                    renderer::present();
                });

                frame_end(&mut pre_render_systems);
                frame_end(&mut render_systems);
            }

            None => break 'game_loop,
        }

        let elapsed = frame_start.elapsed();
        info!("Frame time {} ms", ((elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0) * 1000.0) as u32);

        if frame_time > elapsed {
            std::thread::sleep(frame_time - elapsed);
        }
    }
}

fn start_entity(systems: &mut Vec<Box<System>>, entity: Entity) {
    for system in systems.iter_mut() {
        if !entity.disabled() {
            system.start(entity);
        }
    }

    for child in entity.children().into_iter() {
        start_entity(systems, child);
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
    for system in systems.iter_mut() {
        if !entity.disabled() {
            system.update(entity);
        }
    }

    for child in entity.children().into_iter() {
        update_entity(systems, child);
    }
}

fn post_update_entity(systems: &mut Vec<Box<System>>, entity: Entity) {
    for system in systems.iter_mut() {
        if !entity.disabled() {
            system.post_update(entity);
        }
    }

    for child in entity.children().into_iter() {
        post_update_entity(systems, child);
    }
}
