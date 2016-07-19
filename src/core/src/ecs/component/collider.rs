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

    pub fn test(&self, other: &Collider) -> Option<String> {
        for group in self.group.iter() {
            if let Some(group) = other.mask.iter().find(|&mask| group == mask) {
                return Some(group.to_string());
            }
        }

        None
    }
}
