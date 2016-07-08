use std::any::Any;
use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};

use math::Transform;

pub use self::sprite::Sprite;
pub use self::animator::Animator;
pub use self::camera::Camera;

pub mod sprite;
pub mod camera;
pub mod animator;

pub trait Component: Any + Clone + Default {}

pub struct ComponentRef<C: Component> {
    component: Rc<RefCell<C>>,
}

impl<C: Component> ComponentRef<C> {
    pub fn new(component: C) -> ComponentRef<C> {
        ComponentRef {
            component: Rc::new(RefCell::new(component)),
        }
    }

    pub fn read(&self) -> Ref<C> {
        self.component.borrow()
    }

    pub fn write(&self) -> RefMut<C> {
        self.component.borrow_mut()
    }
}

impl<C: Component> Clone for ComponentRef<C> {
    fn clone(&self) -> ComponentRef<C> {
        ComponentRef {
            component: self.component.clone(),
        }
    }
}

impl Component for Transform {}
