use std::path::Path;

use super::*;
use super::image::Image;

use math::*;

use Error;

pub struct Sprite {
    image: Asset<Image>,
    region: Rect,
    anchor: Vec2,
}

unsafe impl Send for Sprite {}

impl Resource for Sprite {
    fn type_name() -> &'static str {
        "Sprite"
    }
}

impl Sprite {
    pub fn image(&self) -> &Asset<Image> {
        &self.image
    }

    pub fn region(&self) -> &Rect {
        &self.region
    }

    pub fn anchor(&self) -> &Vec2 {
        &self.anchor
    }
}

impl Loadable<Asset<Image>> for Sprite {
    fn load(image: Asset<Image>) -> Result<Sprite, Error> {
        let size = try!(image.access(|image| {
            image.size()
        }));
        Ok(Sprite {
            image: image,
            region: Rect::with_min_max(vec2(0.0, 0.0), vec2(size.0 as f32, size.1 as f32)),
            anchor: vec2(0.0, 0.0),
        })
    }
}

impl<'a> Loadable<&'a Asset<Image>> for Sprite {
    fn load(image: &'a Asset<Image>) -> Result<Sprite, Error> {
        Loadable::load(image.clone())
    }
}

impl<P: AsRef<Path>> Loadable<P> for Sprite {
    fn load(path: P) -> Result<Sprite, Error> {
        let image = Asset::<Image>::new("".to_string());
        try!(image.load(path));
        Loadable::load(image)
    }
}
