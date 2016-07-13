use std::rc::Rc;
use std::cell::RefCell;

use super::System;

use ecs::Entity;
use ecs::component::Component;

pub struct BehaviourSystem {}

impl System for BehaviourSystem {
    fn start(&mut self, entity: Entity) {
        if let Some(behaviour_delegate) = entity.component::<BehaviourDelegate>() {
            behaviour_delegate.write().start(entity);
        }
    }

    fn update(&mut self, entity: Entity) {
        if let Some(behaviour_delegate) = entity.component::<BehaviourDelegate>() {
            behaviour_delegate.write().update(entity);
        }
    }

    fn post_update(&mut self, entity: Entity) {
        if let Some(behaviour_delegate) = entity.component::<BehaviourDelegate>() {
            behaviour_delegate.write().post_update(entity);
        }
    }
}

pub trait Behaviour {
    fn start(&mut self, _entity: Entity) {}
    fn update(&mut self, _entity: Entity) {}
    fn post_update(&mut self, _entity: Entity) {}
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

    pub fn start(&mut self, entity: Entity) {
        if let Some(ref behaviour) = self.behaviour {
            behaviour.borrow_mut().start(entity);
        }
    }

    pub fn update(&mut self, entity: Entity) {
        if let Some(ref behaviour) = self.behaviour {
            behaviour.borrow_mut().update(entity);
        }
    }

    pub fn post_update(&mut self, entity: Entity) {
        if let Some(ref behaviour) = self.behaviour {
            behaviour.borrow_mut().post_update(entity);
        }
    }
}
