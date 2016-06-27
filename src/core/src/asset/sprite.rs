use std::sync::{Arc, RwLock, RwLockReadGuard};

use super::*;
use super::image::Image;

use math::*;

#[derive(Clone)]
pub struct Sprite {
    content: Arc<RwLock<Content>>,
}

impl Sprite {
    pub fn new(image: &Image, region: Rect, anchor: Vec2) -> Sprite {
        Sprite {
            content: Arc::new(RwLock::new(Content {
                image: image.clone(),
                region: region,
                anchor: anchor,
            })),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<Content> {
        self.content.read().unwrap()
    }
}

pub struct Content {
    image: Image,
    region: Rect,
    anchor: Vec2,
}

impl Content {
    pub fn image(&self) -> &Image {
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
