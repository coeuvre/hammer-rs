use super::Event;

use ecs::Entity;

pub struct Collision {
    pub other: Entity,
    pub group: String,
}

impl Event for Collision {}
