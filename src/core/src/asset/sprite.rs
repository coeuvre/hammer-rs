use super::*;

use math::*;

pub type SpriteRef = AssetRef<Sprite>;

pub struct Sprite {
    image: ImageRef,
    region: Rect,
    anchor: Vec2,
}

impl Sprite {
    pub fn new(image: &ImageRef, region: Rect, anchor: Vec2) -> Sprite {
        Sprite {
            image: image.clone(),
            region: region,
            anchor: anchor,
        }
    }

    pub fn image(&self) -> &ImageRef {
        &self.image
    }

    pub fn region(&self) -> &Rect {
        &self.region
    }

    pub fn anchor(&self) -> &Vec2 {
        &self.anchor
    }
}

impl Asset for Sprite {
    fn name() -> &'static str {
        "Sprite"
    }
}
