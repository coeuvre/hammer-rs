use std::any::Any;

pub use self::animation_finished::AnimationFinished;
pub use self::collision::Collision;

mod animation_finished;
mod collision;

pub trait Event: Any {}
