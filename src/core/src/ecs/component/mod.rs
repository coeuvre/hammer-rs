use std::any::Any;

use math::Transform;

pub use self::sprite::Sprite;
pub use self::animator::Animator;
pub use self::camera::Camera;

pub mod sprite;
pub mod camera;
pub mod animator;

pub trait Component: Any {}

impl Component for Transform {}
