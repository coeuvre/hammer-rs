use super::System;

use ecs::Entity;

pub struct BehaviourSystem {}

impl System for BehaviourSystem {
    fn start(&mut self, entity: Entity) {
        entity.start();
    }

    fn update(&mut self, entity: Entity) {
        entity.update();
    }

    fn post_update(&mut self, entity: Entity) {
        entity.post_update();
    }
}
