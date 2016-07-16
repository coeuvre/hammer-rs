use std::collections::HashMap;
use std::marker::PhantomData;
use std::cell::RefCell;
use std::rc::Rc;

use typemap::{TypeMap, Key};

use Error;

use math::*;

use util::counter::Counter;

use self::component::Component;

pub mod component;
pub mod system;

thread_local!(static WORLD: World = World::new());

thread_local!(static COUNTER: Counter<usize> = Counter::new(0));

pub trait Event: Key {}

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct Entity(usize);

impl Entity {
    pub fn new<S: Into<String>>(id: S) -> Entity {
        let entity = Entity(COUNTER.with(|counter| counter.next()));
        let storage = EntityStorage::new(entity, id.into());
        WORLD.with(|world| world.insert_entity(entity, storage));
        entity
    }

    pub fn destroy(self) {
        WORLD.with(|world| world.destroy_entity(self))
    }

    fn get(&self) -> Option<Rc<EntityStorage>> {
        WORLD.with(|world| world.get_entity(self))
    }

    pub fn add_component<C: Component>(&self, component: C) {
        self.get().map(|entity| entity.add_component(component));
    }

    pub fn add_child(&self, child: Entity) -> Result<(), Error> {
        if let Some(this) = self.get() {
            this.add_child(child)
        } else {
            Err("Entity does not exists.".into())
        }
    }

    pub fn id(&self) -> String {
        self.get().map(|entity| entity.id().to_string()).unwrap_or("".to_string())
    }

    pub fn parent(&self) -> Option<Entity> {
        self.get().and_then(|entity| entity.parent())
    }

    pub fn with<F, R, C>(&self, f: F) -> Option<R> where F: FnOnce(&C) -> R, C: Component {
        self.get().and_then(|entity| entity.with(f))
    }

    pub fn with_mut<F, R, C>(&self, f: F) -> Option<R> where F: FnOnce(&mut C) -> R, C: Component {
        self.get().and_then(|entity| entity.with_mut(f))
    }

    pub fn children(&self) -> Vec<Entity> {
        self.get().map(|entity| entity.children()).unwrap_or(Vec::new())
    }

    pub fn on_start<F: Fn(Entity) + 'static>(&self, handler: F) {
        self.get().map(|entity| entity.on_start(handler));
    }

    pub fn start(&self) {
        self.get().map(|entity| entity.start());
    }

    pub fn on_update<F: Fn(Entity) + 'static>(&self, handler: F) {
        self.get().map(|entity| entity.on_update(handler));
    }

    pub fn update(&self) {
        self.get().map(|entity| entity.update());
    }

    pub fn on_post_update<F: Fn(Entity) + 'static>(&self, handler: F) {
        self.get().map(|entity| entity.on_post_update(handler));
    }

    pub fn post_update(&self) {
        self.get().map(|entity| entity.post_update());
    }

    pub fn on_event<E: Event, F: Fn(Entity, &E, Entity) + 'static>(&self, handler: F) {
        self.get().map(|entity| entity.on_event(handler));
    }

    pub fn send<E: Event>(&self, event: E, recv: Entity) {
        recv.get().map(|recv| recv.recv(event, *self));
    }

    pub fn transform_to_world(&self) -> Transform {
        self.get().map(|entity| entity.transform_to_world()).unwrap_or(Transform::identity())
    }
}


struct EntityStorage {
    entity: Entity,

    id: String,
    parent: RefCell<Option<Entity>>,
    children: RefCell<Vec<Entity>>,
    components: RefCell<TypeMap>,

    start_handlers: RefCell<Vec<Box<Fn(Entity)>>>,
    update_handlers: RefCell<Vec<Box<Fn(Entity)>>>,
    post_update_handlers: RefCell<Vec<Box<Fn(Entity)>>>,

    event_handlers: RefCell<TypeMap>,
}

impl EntityStorage {
    pub fn new(entity: Entity, id: String) -> EntityStorage {
        EntityStorage {
            entity: entity,

            id: id,
            parent: RefCell::new(None),
            children: RefCell::new(Vec::new()),
            components: RefCell::new(TypeMap::new()),

            start_handlers: RefCell::new(Vec::new()),
            update_handlers: RefCell::new(Vec::new()),
            post_update_handlers: RefCell::new(Vec::new()),

            event_handlers: RefCell::new(TypeMap::new()),
        }
    }

    pub fn add_child(&self, entity: Entity) -> Result<(), Error> {
        if let Some(storage) = entity.get() {
            let child_id = entity.id();
            if self.has_child(&child_id) {
                Err(format!("Entity with id {} already exists.", child_id).into())
            } else {
                *storage.parent.borrow_mut() = Some(self.entity);
                self.children.borrow_mut().push(entity);
                Ok(())
            }
        } else {
            Err("Entity does not exists.".into())
        }
    }

    fn has_child(&self, id: &str) -> bool {
        for child in self.children.borrow().iter() {
            if child.id() == id {
                return true;
            }
        }

        false
    }

    pub fn add_component<C: Component>(&self, component: C) {
        self.components.borrow_mut().insert::<ComponentTypeMapKey<C>>(Rc::new(RefCell::new(component)));
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn parent(&self) -> Option<Entity> {
        *self.parent.borrow()
    }

    pub fn children(&self) -> Vec<Entity> {
        self.children.borrow().clone()
    }

    pub fn with<F, R, C>(&self, f: F) -> Option<R> where F: FnOnce(&C) -> R, C: Component {
        let component = self.components.borrow().get::<ComponentTypeMapKey<C>>().cloned();
        component.map(|component| f(&*component.borrow()))
    }

    pub fn with_mut<F, R, C>(&self, f: F) -> Option<R> where F: FnOnce(&mut C) -> R, C: Component {
        let component = self.components.borrow().get::<ComponentTypeMapKey<C>>().cloned();
        component.map(|component| f(&mut *component.borrow_mut()))
    }

    pub fn on_start<F: Fn(Entity) + 'static>(&self, handler: F) {
        self.start_handlers.borrow_mut().push(Box::new(handler));
    }

    pub fn start(&self) {
        for handler in self.start_handlers.borrow().iter() {
            handler(self.entity);
        }
    }

    pub fn on_update<F: Fn(Entity) + 'static>(&self, handler: F) {
        self.update_handlers.borrow_mut().push(Box::new(handler));
    }

    pub fn update(&self) {
        for handler in self.update_handlers.borrow().iter() {
            handler(self.entity);
        }
    }

    pub fn on_post_update<F: Fn(Entity) + 'static>(&self, handler: F) {
        self.post_update_handlers.borrow_mut().push(Box::new(handler));
    }

    pub fn post_update(&self) {
        for handler in self.post_update_handlers.borrow().iter() {
            handler(self.entity);
        }
    }

    pub fn on_event<E: Event, F: Fn(Entity, &E, Entity) + 'static>(&self, handler: F) {
        let mut event_handlers = self.event_handlers.borrow_mut();
        let mut handlers = event_handlers.entry::<EventTypeMapKey<E>>().or_insert(Vec::new());
        handlers.push(Box::new(handler));
    }

    pub fn recv<E: Event>(&self, event: E, send: Entity) {
        let event_handlers = self.event_handlers.borrow();
        if let Some(handlers) = event_handlers.get::<EventTypeMapKey<E>>() {
            for handler in handlers.iter() {
                handler(self.entity, &event, send);
            }
        }
    }

    pub fn transform_to_world(&self) -> Transform {
        let transform = self.with(|trans| *trans).unwrap_or(Transform::identity());
        let parent = self.parent().map(|parent| parent.transform_to_world());
        parent.map(|parent| transform * parent).unwrap_or(transform)
    }
}

struct ComponentTypeMapKey<C: Component> {
    phantom: PhantomData<C>,
}

impl<C: Component> Key for ComponentTypeMapKey<C> {
    type Value = Rc<RefCell<C>>;
}

struct EventTypeMapKey<E: Event> {
    phantom: PhantomData<E>,
}

impl<E: Event> Key for EventTypeMapKey<E> {
    type Value = Vec<Box<Fn(Entity, &E, Entity)>>;
}

struct World {
    entities: RefCell<HashMap<Entity, Rc<EntityStorage>>>,
}

impl World {
    pub fn new() -> World {
        World {
            entities: RefCell::new(HashMap::new()),
        }
    }

    pub fn insert_entity(&self, entity: Entity, storage: EntityStorage) {
        let mut entities = self.entities.borrow_mut();
        entities.insert(entity, Rc::new(storage));
    }

    pub fn get_entity(&self, entity: &Entity) -> Option<Rc<EntityStorage>> {
        let entities = self.entities.borrow();
        entities.get(entity).cloned()
    }

    pub fn destroy_entity(&self, entity: Entity) {
        if let Some(parent) = entity.parent() {
            if let Some(parent) = parent.get() {
                let mut children = parent.children.borrow_mut();
                *children = children.clone().into_iter().filter(|&child| child != entity).collect();
            }
        }

        self.destroy_child(entity);
    }

    fn destroy_child(&self, entity: Entity) {
        let storage = self.entities.borrow_mut().remove(&entity);

        if let Some(storage) = storage {
            for child in storage.children().into_iter() {
                self.destroy_child(child);
            }
        }
    }
}
