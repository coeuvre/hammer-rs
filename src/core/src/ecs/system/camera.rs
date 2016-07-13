use super::System;

use ecs::Entity;
use ecs::component::Camera;

use renderer;

pub struct CameraSystem {}

impl System for CameraSystem {
    fn frame_begin(&mut self) {
        renderer::clear_camera();
    }

    fn post_update(&mut self, entity: Entity) {
        if let Some(camera) = entity.component::<Camera>() {
            let trans = entity.transform_to_world();
            let camera = camera.read();
            let mut render_camera = renderer::RenderCamera::new(*camera.region());
            render_camera.set_transform(trans.invert());
            render_camera.set_background(camera.background());
            renderer::add_camera(render_camera);
        }
    }
}

impl CameraSystem {
    pub fn new() -> CameraSystem {
        CameraSystem {
        }
    }
}
