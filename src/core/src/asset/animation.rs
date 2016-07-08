use super::*;

pub type AnimationRef = AssetRef<Animation>;

#[derive(Copy, Clone)]
pub enum WrapMode {
    Loop,
    Once,
}

pub struct Animation {
    frames: Vec<Frame>,
    fps: u16,
    wrap_mode: WrapMode,
}

impl Asset for Animation {
    fn name() -> &'static str {
        "Animation"
    }
}

impl Animation {
    pub fn new() -> Animation {
        Animation {
            frames: Vec::new(),
            fps: 60,
            wrap_mode: WrapMode::Loop,
        }
    }

    pub fn add_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    pub fn frames(&self) -> &[Frame] {
        self.frames.as_slice()
    }

    pub fn wrap_mode(&self) -> WrapMode {
        self.wrap_mode
    }

    pub fn fps(&self) -> u16 {
        self.fps
    }
}
