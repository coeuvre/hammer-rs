use super::Component;

use math::Rect;

#[derive(Clone, Default)]
pub struct Collider {
    pub shape: Rect,
    mask: Vec<String>,
    group: Vec<String>,
}

impl Component for Collider {}

impl Collider {
    pub fn new<S: Into<String>>(shape: Rect, mask: Vec<S>, group: Vec<S>) -> Collider {
        Collider {
            shape: shape,
            mask: mask.into_iter().map(|s| s.into()).collect(),
            group: group.into_iter().map(|s| s.into()).collect(),
        }
    }

    pub fn test(&self, other: &Collider) -> bool {
        for group in self.group.iter() {
            if let Some(_) = other.mask.iter().position(|mask| group == mask) {
                return true;
            }
        }

        false
    }
}
