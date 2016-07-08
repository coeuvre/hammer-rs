use super::System;

use ecs::EntityRef;
use ecs::component::Sprite;

use math::*;
use renderer;
use renderer::Drawable;

pub struct SpriteSystem {}

impl System for SpriteSystem {
    fn update(&mut self, entity: &EntityRef) {
        if let Some(sprite) = entity.component::<Sprite>() {
            let trans = entity.transform_to_world();
            renderer::set_transform(trans);
            let sprite = sprite.read();
            if let Some(frame) = sprite.frame() {
                let anchor = sprite.anchor() % frame.region().size();
                renderer::rect(Rect::with_min_size(-anchor, frame.region().size())).texture(frame).draw();
            }
        }
    }
}
