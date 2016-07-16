use super::System;

use ecs::Entity;
use ecs::component::{Collider};
use ecs::event::Collision;

pub struct CollisionSystem {
}

impl CollisionSystem {
    pub fn new() -> CollisionSystem {
        CollisionSystem {}
    }
}

impl System for CollisionSystem {
    fn update(&mut self, entity: Entity) {
        let mut collided = None;

        entity.with(|collider: &Collider| {
            for other in Entity::all() {
                if entity != other {
                    other.with(|other_collider: &Collider| {
                        if collider.test(other_collider) {
                            let this_offset = entity.transform_to_world().position();
                            let other_offset = other.transform_to_world().position();

                            let this_rect = collider.shape.offset(this_offset);
                            let other_rect = other_collider.shape.offset(other_offset);

                            if let Some(_) = this_rect.intersect(&other_rect) {
                                collided = Some(other);
                            }
                        }
                    });
                }
            }
        });

        if let Some(other) = collided {
            other.send(Collision {}, entity);
        }
    }
}
