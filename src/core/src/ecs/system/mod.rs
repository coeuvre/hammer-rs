pub use self::animation::AnimationSystem;
pub use self::camera::CameraSystem;
pub use self::behaviour::BehaviourSystem;

pub use self::sprite::SpriteSystem;

pub mod animation;
pub mod camera;
pub mod behaviour;

pub mod sprite;

use ecs::Entity;

pub trait System {
    fn start(&mut self, _entity: Entity) {}

    fn frame_begin(&mut self) {}
    fn update(&mut self, _entity: Entity) {}
    fn post_update(&mut self, _entity: Entity) {}
    fn frame_end(&mut self) {}
}
