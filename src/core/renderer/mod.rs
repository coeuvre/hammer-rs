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
    fn draw(&self);
}

pub struct Trans {
    trans: Transform,
}

impl Trans {
    pub fn rect(self, rect: Rect) -> Quad {
        Quad {
            rect: rect,
            trans: self.trans,
        }
    }

    pub fn texture<T: gl::AsTexture>(self, texture: &T) -> TexturedQuad<T> {
        TexturedQuad {
            texture: texture.clone(),
            dst: None,
            trans: self.trans,
        }
    }
}

pub struct Quad {
    rect: Rect,
    trans: Transform,
}

impl Quad {
    pub fn texture<T: gl::AsTexture>(self, texture: &T) -> TexturedQuad<T> {
        TexturedQuad {
            texture: texture.clone(),
            dst: Some(self.rect),
            trans: self.trans,
        }
    }

    pub fn color(self, mut r: f32, mut g: f32, mut b: f32, a: f32) -> ColoredQuad {
        // gamma correction and pre-multiply alpha
        let gamma = 2.1;

        r = r.powf(gamma);
        g = g.powf(gamma);
        b = b.powf(gamma);

        r = r * a;
        g = g * a;
        b = b * a;

        ColoredQuad {
            color: (r, g, b, a),
            dst: self.rect,
            trans: self.trans,
        }
    }
}

pub struct TexturedQuad<T> {
    texture: T,
    dst: Option<Rect>,
    trans: Transform,
}

impl<T: gl::AsTexture + 'static> Drawable for TexturedQuad<T> {
    fn push(self, order: RenderOrder) {
        CONTEXT.with(|context| {
            context.add_drawable(self, order);
        });
    }

    fn draw(&self) {
        CONTEXT.with(|context| {
            if let Some(ref mut renderer) = *context.renderer.borrow_mut() {
                renderer.fill_with_texture(*context.projection.borrow() * self.trans, self.dst.as_ref(), &self.texture);
            }
        });
    }
}

pub struct ColoredQuad {
    color: (f32, f32, f32, f32),
    dst: Rect,
    trans: Transform
}

impl Drawable for ColoredQuad {
    fn push(self, order: RenderOrder) {
        CONTEXT.with(|context| {
            context.add_drawable(self, order);
        });
    }

    fn draw(&self) {
        CONTEXT.with(|context| {
            if let Some(ref mut renderer) = *context.renderer.borrow_mut() {
                renderer.fill_with_color(*context.projection.borrow() * self.trans, &self.dst,
                                         self.color.0, self.color.1, self.color.2, self.color.3);
            }
        });
    }
}

struct Context {
    renderer: RefCell<Option<Renderer>>,
    cameras: RefCell<Vec<RenderCamera>>,

    seq: Cell<usize>,
    drawables: RefCell<Vec<(RenderOrder, Box<Drawable>)>>,

    projection: RefCell<Transform>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            renderer: RefCell::new(None),
            cameras: RefCell::new(Vec::new()),
            seq: Cell::new(0),
            drawables: RefCell::new(Vec::new()),
            projection: RefCell::new(Transform::identity()),
        }
    }

    pub fn set_target(&self, window: &Window) {
        *self.renderer.borrow_mut() = Renderer::new(window).ok();
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
        let mut drawables = self.drawables.borrow_mut();
        drawables.sort_by(|a, b| a.0.cmp(&b.0));

        for (_, drawable) in drawables.drain(..) {
            drawable.draw();
        }

        if let Some(ref mut renderer) = *self.renderer.borrow_mut() {
            renderer.present();
        }

        self.seq.set(0);
    }

    pub fn trans(&self, trans: Transform) -> Trans {
        Trans {
            trans: trans,
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

pub fn set_projection(trans: Transform) {
    CONTEXT.with(|context| context.set_projection(trans))
}

pub fn rect(rect: Rect) -> Quad {
    CONTEXT.with(|context| context.trans(Transform::identity()).rect(rect))
}

pub fn texture<T: gl::AsTexture>(texture: &T) -> TexturedQuad<T> {
    CONTEXT.with(|context| context.trans(Transform::identity()).texture(texture))
}

pub fn trans(trans: Transform) -> Trans {
    CONTEXT.with(|context| context.trans(trans))
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
