use super::System;

use ecs::Entity;
use ecs::component::Sprite;

use math::*;
use renderer;
use renderer::{Drawable, RenderOrder};

pub struct SpriteSystem {
}

impl SpriteSystem {
    pub fn new() -> SpriteSystem {
        SpriteSystem {
        }
    }
}

impl System for SpriteSystem {
    fn update(&mut self, entity: Entity) {
        entity.with(|sprite: &Sprite| {
            let trans = entity.transform_to_world();
            renderer::set_transform(trans);
            if let Some(frame) = sprite.frame() {
                let anchor = frame.anchor() % frame.region().size();
                let order = RenderOrder::new(sprite.layer(), sprite.order());
                renderer::rect(Rect::with_min_size(-anchor, frame.region().size())).texture(frame).push(order);
            }
        });
    }
}
