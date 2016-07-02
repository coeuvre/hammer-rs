use std::any::Any;

use asset::{asset, Image, ImageRef};
use math::*;
use input::Input;
use renderer::{Renderer, Drawable};

pub trait AsScene {
    fn as_scene(&self) -> &Scene;
    fn as_scene_mut(&mut self) -> &mut Scene;

    fn start(&mut self);
    fn update(&mut self, input: &Input);
}

pub struct Scene {
    name: String,
    root: Box<AsNode>,
}

impl Scene {
    pub fn new<T: AsNode>(name: String, root: T) -> Scene {
        Scene {
            name: name,
            root: Box::new(root),
        }
    }

    pub fn add_child<T: AsNode + 'static>(&mut self, child: T) {
        self.root.as_node_mut().add_child(child)
    }

    pub fn root(&self) -> &AsNode {
        &*self.root
    }
}

pub struct Node {
    name: String,
    trans: Trans,
    children: Vec<Box<AsNode>>,
}

pub trait Transform {
    fn trans(&self) -> &Trans;
    fn set_position(&mut self, x: Scalar, y: Scalar);
}

impl<T: AsNode> Transform for T {
    fn trans(&self) -> &Trans {
        &self.as_node().trans
    }

    fn set_position(&mut self, x: Scalar, y: Scalar) {
        self.as_node_mut().trans.set_origin(x, y);
    }
}

impl Node {
    pub fn new(name: String) -> Node {
        Node {
            name: name,
            trans: Trans::identity(),
            children: Vec::new(),
        }
    }

    pub fn add_child<T: AsNode + 'static>(&mut self, child: T) {
        self.children.push(Box::new(child))
    }

    pub fn children(&self) -> ::std::slice::Iter<Box<AsNode>> {
        self.children.iter()
    }
}

impl AsNode for Node {
    fn as_node(&self) -> &Node {
        self
    }

    fn as_node_mut(&mut self) -> &mut Node {
        self
    }

    fn render(&self, _renderer: &mut Renderer) {}
}

pub trait AsNode: Any {
    fn as_node(&self) -> &Node;
    fn as_node_mut(&mut self) -> &mut Node;

    fn render(&self, renderer: &mut Renderer);
}

pub struct Sprite {
    node: Node,

    image: ImageRef,
    region: Rect,

    anchor: Vec2,
}

impl Sprite {
    pub fn new(path: &str) -> Sprite {
        let image = asset::<Image>::new().load(path).unwrap();
        let (w, h) = image.read().size();
        Sprite {
            node: Node::new("Sprite".to_string()),
            image: image,
            region: Rect::with_min_size(vec2(0.0, 0.0), vec2(w as f32, h as f32)),
            anchor: vec2(0.0, 0.0),
        }
    }

    pub fn name(&self) -> &str {
        self.node.name.as_str()
    }
}

impl AsNode for Sprite {
    fn as_node(&self) -> &Node {
        &self.node
    }

    fn as_node_mut(&mut self) -> &mut Node {
        &mut self.node
    }

    fn render(&self, renderer: &mut Renderer) {
        renderer.rect(Rect::with_min_size(vec2(0.0, 0.0), self.region.size())).texture(&self.image).draw();
    }
}
