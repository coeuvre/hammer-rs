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
        for event in window.poll_events() {
            match event {
                window::Event::Close => break 'game_loop,
                _ => {}
            }
        }

        renderer::clear(0.0, 0.0, 0.0, 1.0);

        if let Some(scene) = scene::top() {
            if !started_scene.contains(scene.read().id()) {
                start_entity(&mut systems, scene.read().root());
                started_scene.insert(scene.read().id().to_string());
            }

            update_entity(&mut systems, scene.read().root());
        }

        renderer::present();
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

/*
            let view_w = 640.0;
            let view_h = 480.0;

            self.renderer.ortho(0.0, view_w, 0.0, view_h);

            self.renderer.clear(0.0, 0.0, 0.0, 1.0);

            self.process_entity(scene.root());

            self.renderer.present();


        if let Some(orig_trans) = orig_trans {
            self.renderer.set_trans(orig_trans);
        }
        */
