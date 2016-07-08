pub use self::camera::CameraSystem;
pub use self::behaviour::BehaviourSystem;
pub use self::sprite::SpriteSystem;

pub mod camera;
pub mod behaviour;

pub mod sprite;

use ecs::EntityRef;

pub trait System {
    fn start(&mut self, _entity: &EntityRef) {}

    fn frame_begin(&mut self) {}
    fn update(&mut self, _entity: &EntityRef) {}
    fn post_update(&mut self, _entity: &EntityRef) {}
    fn frame_end(&mut self) {}
}
