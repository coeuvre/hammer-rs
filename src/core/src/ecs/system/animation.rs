use super::System;

use ecs::Entity;
use ecs::component::{Animator, Sprite};
use ecs::event::AnimationFinished;

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
        let mut finished = false;
        entity.with_mut(|animator: &mut Animator| {
            animator.advance(input::delta());

            entity.with_mut(|sprite: &mut Sprite| {
                sprite.set_frame(animator.current_frame());
            });

            if animator.finished() {
                finished = true;
            }
        });

        if finished {
            entity.send(AnimationFinished {}, entity);
        }
    }
}
