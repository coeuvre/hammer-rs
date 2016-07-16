use std::any::Any;

use math::Transform;

pub use self::sprite::Sprite;
pub use self::animator::Animator;
pub use self::camera::Camera;
pub use self::collider::Collider;

mod sprite;
mod camera;
mod animator;
mod collider;

pub trait Component: Any {}

impl Component for Transform {}
