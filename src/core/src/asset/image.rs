use std::path::Path;
use std::ptr;
use std::ffi::CString;

use super::*;

use Error;

use math::{Rect, Vector};

use util::stb_image::*;
use util::cstr_to_string;

use util::counter::Counter;

lazy_static! {
    static ref COUNTER: Counter<usize> = Counter::new(0);
}

pub type ImageRef = AssetRef<Image>;

pub struct Image {
    id: usize,
    w: i32,
    h: i32,
    data: Vec<u8>,
}

impl Image {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn size(&self) -> (i32, i32) {
        (self.w, self.h)
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl Asset for Image {
    fn name() -> &'static str {
        "Image"
    }
}

impl Loadable for Image {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();
        unsafe {
            let cstr = CString::new(&*path.as_os_str().to_string_lossy()).unwrap();
            let mut w = 0;
            let mut h = 0;
            let data = stbi_load(cstr.as_ptr(), &mut w, &mut h, ptr::null_mut(), 4);
            if data != ptr::null_mut() {
                let mut pixels = Vec::with_capacity((w * 4 * h) as usize);
                // Flip the pixels
                {
                    let data = ::std::slice::from_raw_parts(data, (w * 4 * h) as usize);
                    for row in data.chunks((w * 4) as usize).rev() {
                        for pixel in row.chunks(4) {
                            let mut r = pixel[0] as f32 / 255.0;
                            let mut g = pixel[1] as f32 / 255.0;
                            let mut b = pixel[2] as f32 / 255.0;
                            let a = pixel[3] as f32 / 255.0;

                            let gamma = 2.1;

                            r = r.powf(gamma);
                            g = g.powf(gamma);
                            b = b.powf(gamma);

                            r = r * a;
                            g = g * a;
                            b = b * a;

                            r = r.powf(1.0 / gamma);
                            g = g.powf(1.0 / gamma);
                            b = b.powf(1.0 / gamma);

                            pixels.push((r * 255.0) as u8);
                            pixels.push((g * 255.0) as u8);
                            pixels.push((b * 255.0) as u8);
                            pixels.push(pixel[3]);
                        }
                    }
                }
                stbi_image_free(data);
                Ok(Image {
                    id: COUNTER.next(),
                    w: w,
                    h: h,
                    data: pixels,
                })
            } else {
                Err(format!("Failed to load {}: {}", path.display(), cstr_to_string(stbi_failure_reason())).into())
            }
        }
    }
}

#[derive(Clone)]
pub struct Frame {
    image: ImageRef,
    region: Rect,
    anchor: Vector,
}

impl Frame {
    pub fn new(image: ImageRef, region: Rect) -> Frame {
        Frame {
            image: image,
            region: region,
            anchor: Vector::zero(),
        }
    }

    pub fn image(&self) -> &ImageRef {
        &self.image
    }

    pub fn region(&self) -> &Rect {
        &self.region
    }

    pub fn set_region(&mut self, region: Rect) {
        self.region = region;
    }

    pub fn anchor(&self) -> Vector {
        self.anchor
    }

    pub fn set_anchor(&mut self, anchor: Vector) {
        self.anchor = anchor;
    }
}
