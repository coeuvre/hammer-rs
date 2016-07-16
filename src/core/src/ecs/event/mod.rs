use std::any::Any;

pub use self::collision::Collision;

mod collision;

pub trait Event: Any {}
