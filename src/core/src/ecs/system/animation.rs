use super::System;

use ecs::Entity;
use ecs::component::{Animator, Sprite};

use input;

pub struct AnimationSystem {
}

impl AnimationSystem {
    pub fn new() -> AnimationSystem {
        AnimationSystem {}
    }
}

impl System for AnimationSystem {
    fn update(&mut self, entity: Entity) {
        if let Some(animator) = entity.component::<Animator>() {
            animator.write().advance(input::delta());

            if let Some(sprite) = entity.component::<Sprite>() {
                sprite.write().set_frame(animator.read().current_frame());
            }
        }
    }
}
