use std::cell::{Cell, RefCell};

mod gl;

use self::gl::Renderer;

use math::*;
use window::Window;

pub struct RenderCamera {
    region: Rect,
    // viewport: Rect,
    background: (f32, f32, f32, f32),
    transform: Transform,
}

impl RenderCamera {
    pub fn new(region: Rect) -> RenderCamera {
        RenderCamera {
            region: region,
            // viewport: Rect::with_min_size(Vector::zero(), vector(1.0, 1.0)),
            background: (0.0, 0.0, 0.0, 1.0),
            transform: Transform::identity(),
        }
    }

    pub fn transform(&self) -> Transform {
        self.transform
    }

    pub fn set_transform(&mut self, trans: Transform) {
        self.transform = trans;
    }

    pub fn region(&self) -> &Rect {
        &self.region
    }

    pub fn set_region(&mut self, region: Rect) {
        self.region = region;
    }

    pub fn background(&self) -> (f32, f32, f32, f32) {
        self.background
    }

    pub fn set_background(&mut self, background: (f32, f32, f32, f32)) {
        self.background = background;
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct RenderOrder {
    pub layer: i32,
    pub order: i32,
    seq: usize,
}

impl RenderOrder {
    pub fn new(layer: i32, order: i32) -> RenderOrder {
        RenderOrder {
            layer: layer,
            order: order,
            seq: 0,
        }
    }
}

pub trait Drawable {
    fn push(self, order: RenderOrder);
    fn draw(&self, renderer: &mut Renderer);
}

pub struct Quad {
    rect: Rect,
    trans: Transform,
}

impl Quad {
    pub fn texture<T: gl::AsTexture>(self, texture: &T) -> TexturedQuad<T> {
        TexturedQuad {
            texture: texture.clone(),
            dst: self.rect,
            trans: self.trans,
        }
    }
}

pub struct TexturedQuad<T> {
    texture: T,
    dst: Rect,
    trans: Transform,
}

impl<T: gl::AsTexture + 'static> Drawable for TexturedQuad<T> {
    fn push(self, order: RenderOrder) {
        CONTEXT.with(|context| {
            context.add_drawable(self, order);
        });
    }

    fn draw(&self, renderer: &mut Renderer) {
        renderer.fill_with_texture(self.trans, &self.dst, &self.texture);
    }
}

struct Context {
    renderer: RefCell<Option<Renderer>>,
    cameras: RefCell<Vec<RenderCamera>>,

    seq: Cell<usize>,
    drawables: RefCell<Vec<(RenderOrder, Box<Drawable>)>>,

    projection: RefCell<Transform>,
    transform: RefCell<Transform>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            renderer: RefCell::new(None),
            cameras: RefCell::new(Vec::new()),
            seq: Cell::new(0),
            drawables: RefCell::new(Vec::new()),
            projection: RefCell::new(Transform::identity()),
            transform: RefCell::new(Transform::identity()),
        }
    }

    pub fn set_target(&self, window: &Window) {
        *self.renderer.borrow_mut() = Renderer::new(window).ok();
    }

    pub fn set_transform(&self, trans: Transform) {
        *self.transform.borrow_mut() = trans;
    }

    pub fn set_projection(&self, trans: Transform) {
        *self.projection.borrow_mut() = trans;
    }

    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        if let Some(ref mut renderer) = *self.renderer.borrow_mut() {
            renderer.clear(r, g, b, a);
        }
    }

    pub fn present(&self) {
        if let Some(ref mut renderer) = *self.renderer.borrow_mut() {
            {
                let mut drawables = self.drawables.borrow_mut();
                drawables.sort_by(|a, b| a.0.cmp(&b.0));

                for &(_, ref drawable) in drawables.iter() {
                    drawable.draw(renderer);
                }

                drawables.clear();
            }

            renderer.present();
        }
    }

    pub fn rect(&self, rect: Rect) -> Quad {
        Quad {
            rect: rect,
            trans: *self.projection.borrow() * *self.transform.borrow(),
        }
    }

    pub fn add_camera(&self, camera: RenderCamera) {
        self.cameras.borrow_mut().push(camera);
    }

    pub fn clear_camera(&self) {
        self.cameras.borrow_mut().clear();
    }

    pub fn with_camera<F: FnMut(&RenderCamera)>(&self, mut f: F) {
        for camera in self.cameras.borrow().iter() {
            f(camera);
        }
    }

    pub fn add_drawable<T: Drawable + 'static>(&self, drawable: T, mut order: RenderOrder) {
        let seq = self.seq.get();
        order.seq = seq;
        self.seq.set(seq + 1);
        self.drawables.borrow_mut().push((order, Box::new(drawable)));
    }
}

thread_local!(static CONTEXT: Context = Context::new());

pub fn set_target(window: &Window) {
    CONTEXT.with(|context| context.set_target(window))
}

pub fn clear(r: f32, g: f32, b: f32, a: f32) {
    CONTEXT.with(|context| context.clear(r, g, b, a))
}

pub fn present() {
    CONTEXT.with(|context| context.present())
}

pub fn set_transform(trans: Transform) {
    CONTEXT.with(|context| context.set_transform(trans))
}

pub fn set_projection(trans: Transform) {
    CONTEXT.with(|context| context.set_projection(trans))
}

pub fn rect(rect: Rect) -> Quad {
    CONTEXT.with(|context| context.rect(rect))
}

pub fn add_camera(camera: RenderCamera) {
    CONTEXT.with(|context| context.add_camera(camera))
}

pub fn clear_camera() {
    CONTEXT.with(|context| context.clear_camera())
}

pub fn with_camera<F: FnMut(&RenderCamera)>(f: F) {
    CONTEXT.with(|context| context.with_camera(f))
}
