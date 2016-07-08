use asset::{asset, Image, ImageRef, Frame};
use math::*;

use super::Component;

#[derive(Clone, Default)]
pub struct Sprite {
    frame: Option<Frame>,
    layer: i32,
    order: i32,
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
            layer: 0,
            order: 0,
        }
    }

    pub fn empty() -> Sprite {
        Sprite {
            frame: None,
            layer: 0,
            order: 0,
        }
    }

    pub fn with_image(image: ImageRef) -> Sprite {
        let (w, h) = image.read().size();
        let frame = Frame::new(image, Rect::with_min_size(Vector::zero(), vector(w as f32, h as f32)));
        Sprite {
            frame: Some(frame),
            layer: 0,
            order: 0,
        }
    }

    pub fn frame(&self) -> Option<&Frame> {
        self.frame.as_ref()
    }

    pub fn frame_mut(&mut self) -> Option<&mut Frame> {
        self.frame.as_mut()
    }

    pub fn set_frame(&mut self, frame: Option<Frame>) {
        self.frame = frame;
    }

    pub fn layer(&self) -> i32 {
        self.layer
    }

    pub fn set_layer(&mut self, layer: i32) {
        self.layer = layer;
    }

    pub fn order(&self) -> i32 {
        self.order
    }

    pub fn set_order(&mut self, order: i32) {
        self.order = order;
    }
}
