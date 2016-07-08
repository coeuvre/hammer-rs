use asset::{asset, Image, ImageRef, Frame};
use math::*;

use super::Component;

#[derive(Clone, Default)]
pub struct Sprite {
    frame: Option<Frame>,
    anchor: Vector,
}

impl Component for Sprite {}

impl Sprite {
    pub fn new(path: &str) -> Sprite {
        let image = asset::<Image>::load(path).ok();
        let frame = image.map(|image| {
            let (w, h) = image.read().size();
            Frame::new(image, Rect::with_min_size(Vector::zero(), vector(w as f32, h as f32)))
        });
        Sprite {
            frame: frame,
            anchor: vector(0.0, 0.0),
        }
    }

    pub fn with_image(image: ImageRef) -> Sprite {
        let (w, h) = image.read().size();
        let frame = Frame::new(image, Rect::with_min_size(Vector::zero(), vector(w as f32, h as f32)));
        Sprite {
            frame: Some(frame),
            anchor: Vector::zero(),
        }
    }

    pub fn frame(&self) -> Option<&Frame> {
        self.frame.as_ref()
    }

    pub fn frame_mut(&mut self) -> Option<&mut Frame> {
        self.frame.as_mut()
    }

    pub fn anchor(&self) -> Vector {
        self.anchor
    }

    pub fn set_anchor(&mut self, anchor: Vector) {
        self.anchor = anchor;
    }
}
