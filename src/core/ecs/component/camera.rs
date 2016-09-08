use super::Component;

use math::*;

#[derive(Clone, Default)]
pub struct Camera {
    region: Rect,
    // viewport: Rect,
    background: (f32, f32, f32, f32),
}

impl Component for Camera {}

impl Camera {
    pub fn new(region: Rect) -> Camera {
        Camera {
            region: region,
            // viewport: Rect::with_min_size(Vector::zero(), vector(1.0, 1.0)),
            background: (0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn region(&self) -> &Rect {
        &self.region
    }

    pub fn set_region(&mut self, region: Rect) {
        self.region = region;
    }

    pub fn background(&self) -> (f32, f32, f32, f32,) {
        self.background
    }
}
