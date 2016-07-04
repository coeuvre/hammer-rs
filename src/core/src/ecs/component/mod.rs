use std::any::Any;
use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};

use math::Transform;

use super::EntityRef;

pub use self::sprite::Sprite;

pub mod sprite;

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

pub trait Behaviour {
    fn start(&mut self, _entity: &EntityRef) {}
    fn update(&mut self, _entity: &EntityRef) {}
}

#[derive(Clone, Default)]
pub struct BehaviourDelegate {
    behaviour: Option<Rc<RefCell<Behaviour>>>
}

impl Component for BehaviourDelegate {}

impl BehaviourDelegate {
    pub fn new<B: Behaviour + 'static>(behaviour: B) -> BehaviourDelegate {
        BehaviourDelegate {
            behaviour: Some(Rc::new(RefCell::new(behaviour))),
        }
    }

    pub fn start(&mut self, entity: &EntityRef) {
        if let Some(ref behaviour) = self.behaviour {
            behaviour.borrow_mut().start(entity);
        }
    }

    pub fn update(&mut self, entity: &EntityRef) {
        if let Some(ref behaviour) = self.behaviour {
            behaviour.borrow_mut().update(entity);
        }
    }
}
