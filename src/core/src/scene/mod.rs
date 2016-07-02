use asset::{asset, Image, ImageRef};
use math::*;
use input::Input;
use renderer::{Renderer, Drawable};

pub trait NodeOp {
    fn add_child<T: HasNode>(&mut self, child: T);
    fn children(&self) -> ::std::slice::Iter<Box<HasNode>>;
}

pub struct Node {
    name: String,
    children: Vec<Box<HasNode>>,
}

impl Node {
    pub fn new(name: String) -> Node {
        Node {
            name: name,
            children: Vec::new(),
        }
    }

    pub fn add_child<T: HasNode + 'static>(&mut self, child: T) {
        self.children.push(Box::new(child))
    }

    pub fn children(&self) -> ::std::slice::Iter<Box<HasNode>> {
        self.children.iter()
    }
}

pub trait HasNode {
    fn node(&self) -> &Node;
    fn node_mut(&mut self) -> &mut Node;

    fn render(&self, _renderer: &mut Renderer) {}
}

impl HasNode for Node {
    fn node(&self) -> &Node { self }
    fn node_mut(&mut self) -> &mut Node { self}
}

impl<T: HasNode> NodeOp for T {
    fn add_child<N: HasNode + 'static>(&mut self, child: N) {
        self.node_mut().add_child(child);
    }

    fn children(&self) -> ::std::slice::Iter<Box<HasNode>> {
        self.node().children()
    }
}

pub struct Scene {
    name: String,
    root: Box<HasNode>,
}

impl Scene {
    pub fn new<T: HasNode + 'static>(name: String, root: T) -> Scene {
        Scene {
            name: name,
            root: Box::new(root),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn root(&self) -> &HasNode {
        &*self.root
    }
}

pub trait HasScene {
    fn scene(&self) -> &Scene;
    fn scene_mut(&mut self) -> &mut Scene;

    fn start(&mut self) {}
    fn update(&mut self, _input: &Input) {}
}

impl HasScene for Scene {
    fn scene(&self) -> &Scene { self }
    fn scene_mut(&mut self) -> &mut Scene { self }
}

impl<T: HasScene> HasNode for T {
    fn node(&self) -> &Node {
        self.scene().root.node()
    }

    fn node_mut(&mut self) -> &mut Node {
        self.scene_mut().root.node_mut()
    }
}

pub trait HasTrans {
    fn trans(&self) -> &Trans;
    fn trans_mut(&mut self) -> &mut Trans;
}

pub trait TransOp {
    fn set_position(&mut self, x: Scalar, y: Scalar);
}

impl<T: HasTrans> TransOp for T {
    fn set_position(&mut self, x: Scalar, y: Scalar) {
        self.trans_mut().set_origin(x, y);
    }
}

pub struct Sprite {
    node: Node,

    trans: Trans,

    image: ImageRef,
    region: Rect,

    anchor: Vec2,
}

impl Sprite {
    pub fn new(path: &str) -> Sprite {
        let image = asset::<Image>::new().load(path).unwrap();
        Sprite::with_image(image)
    }

    pub fn with_image(image: ImageRef) -> Sprite {
        let (w, h) = image.read().size();
        Sprite {
            node: Node::new("Sprite".to_string()),
            trans: Trans::identity(),
            image: image,
            region: Rect::with_min_size(vec2(0.0, 0.0), vec2(w as f32, h as f32)),
            anchor: vec2(0.0, 0.0),
        }
    }

    pub fn name(&self) -> &str {
        self.node.name.as_str()
    }

    pub fn region(&self) -> &Rect {
        &self.region
    }

    pub fn set_region(&mut self, region: Rect) {
        self.region = region
    }

    pub fn anchor(&self) -> &Vec2 {
        &self.anchor
    }

    pub fn set_anchor(&mut self, x: Scalar, y: Scalar) {
        self.anchor = vec2(x, y);
    }
}

impl HasNode for Sprite {
    fn node(&self) -> &Node { &self.node }
    fn node_mut(&mut self) -> &mut Node { &mut self.node }

    fn render(&self, renderer: &mut Renderer) {
        let trans = self.trans * *renderer.trans();
        renderer.set_trans(trans);
        renderer.rect(Rect::with_min_size(vec2(0.0, 0.0), self.region.size())).texture(&self.image).draw();
    }
}

impl HasTrans for Sprite {
    fn trans(&self) -> &Trans { &self.trans }
    fn trans_mut(&mut self) -> &mut Trans { &mut self.trans }
}
