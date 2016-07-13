use asset::{asset, Animation, AnimationRef, Frame, WrapMode};

use super::Component;

#[derive(Clone, Default)]
pub struct Animator {
    animation: Option<AnimationRef>,
    time: f32,
    frame_index: usize,
}

impl Component for Animator {}

impl Animator {
    pub fn new(id: &str) -> Animator {
        Animator {
            animation: asset::<Animation>::get(id),
            time: 0.0,
            frame_index: 0,
        }
    }

    pub fn play(&mut self, id: &str) {
        self.animation = asset::<Animation>::get(id);
        self.time = 0.0;
        self.frame_index = 0;
    }

    pub fn advance(&mut self, delta: f32) {
        if let Some(ref animation) = self.animation {
            self.time += delta;

            let fps = animation.read().fps();
            let spf = 1.0 / fps as f32;
            while self.time > spf {
                self.time -= spf;
                self.frame_index += 1;
            }

            let frame_len = animation.read().frames().len();
            match animation.read().wrap_mode() {
                WrapMode::Once => {
                    self.frame_index = ::std::cmp::min(self.frame_index, frame_len - 1);
                }
                WrapMode::Loop => {
                    self.frame_index = self.frame_index % frame_len;
                }
            }
        }
    }

    pub fn current_frame(&self) -> Option<Frame> {
        self.animation.as_ref().and_then(|animation| animation.read().frames().get(self.frame_index).cloned())
    }
}
