use asset::{asset, Image, ImageRef};
use math::*;

use super::Component;

#[derive(Clone, Default)]
pub struct Sprite {
    image: Option<ImageRef>,
    region: Rect,
    anchor: Vector,
}

impl Component for Sprite {}

impl Sprite {
    pub fn new(path: &str) -> Sprite {
        let image = asset::<Image>::load(path).ok();
        let (w, h) = image.as_ref().map(|image| image.read().size()).unwrap_or((0, 0));
        Sprite {
            image: image,
            region: Rect::with_min_size(vector(0.0, 0.0), vector(w as f32, h as f32)),
            anchor: vector(0.0, 0.0),
        }
    }

    pub fn with_image(image: ImageRef) -> Sprite {
        let (w, h) = image.read().size();
        Sprite {
            image: Some(image),
            region: Rect::with_min_size(vector(0.0, 0.0), vector(w as f32, h as f32)),
            anchor: vector(0.0, 0.0),
        }
    }

    pub fn image(&self) -> Option<&ImageRef> {
        if let Some(ref image) = self.image {
            Some(image)
        } else {
            None
        }
    }

    pub fn region(&self) -> &Rect {
        &self.region
    }

    pub fn set_region(&mut self, region: Rect) {
        self.region = region
    }

    pub fn anchor(&self) -> &Vec2 {
        &self.anchor
    }

    pub fn set_anchor(&mut self, x: Scalar, y: Scalar) {
        self.anchor = vec2(x, y);
    }
}
