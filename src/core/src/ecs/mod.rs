use std::collections::HashMap;
use std::marker::PhantomData;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::any::TypeId;
use std::mem;

use typemap::{TypeMap, Key};

use Error;

use math::*;

use util::counter::Counter;

use self::component::Component;
use self::event::Event;

pub mod component;
pub mod system;
pub mod event;

thread_local!(pub static WORLD: World = World::new());

thread_local!(static COUNTER: Counter<usize> = Counter::new(0));

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct Entity(usize);

impl Entity {
    pub fn new<S: Into<String>>(id: S) -> Entity {
        let entity = Entity(COUNTER.with(|counter| counter.next()));
        let storage = EntityStorage::new(entity, id.into());
        WORLD.with(|world| world.insert_entity(entity, storage));
        entity
    }

    pub fn all() -> Vec<Entity> {
        WORLD.with(|world| world.all_entities())
    }

    pub fn destroy(self) {
        WORLD.with(|world| world.destroy_entity(self))
    }

    pub fn disable(self) {
        WORLD.with(|world| world.disable_entity(self))
    }

    pub fn disabled(self) -> bool {
        self.get().map(|entity| entity.disabled.get()).unwrap_or(true)
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
        recv.get().map(|recv| {
            let ty = TypeId::of::<E>();
            recv.recv_by_type(ty, unsafe { mem::transmute(&event) }, *self)
        });
    }

    pub fn send_after<E: Event>(&self, event: E, recv: Entity, time: Scalar) {
        let event = SentEvent::new(*self, event, recv, time);
        WORLD.with(|world| world.send_event(event))
    }

    pub fn send_by_type(&self, ty: TypeId, event: *const (), recv: Entity) {
        recv.get().map(|recv| recv.recv_by_type(ty, event, *self));
    }

    pub fn transform_to_world(&self) -> Transform {
        self.get().map(|entity| entity.transform_to_world()).unwrap_or(Transform::identity())
    }
}


struct EntityStorage {
    entity: Entity,

    disabled: Cell<bool>,

    id: String,
    parent: RefCell<Option<Entity>>,
    children: RefCell<Vec<Entity>>,
    components: RefCell<TypeMap>,

    start_handlers: RefCell<Vec<Box<Fn(Entity)>>>,
    update_handlers: RefCell<Vec<Box<Fn(Entity)>>>,
    post_update_handlers: RefCell<Vec<Box<Fn(Entity)>>>,

    event_handlers: RefCell<HashMap<TypeId, Vec<Box<Fn(Entity, *const (), Entity)>>>>,
}

impl EntityStorage {
    pub fn new(entity: Entity, id: String) -> EntityStorage {
        EntityStorage {
            entity: entity,

            disabled: Cell::new(false),

            id: id,
            parent: RefCell::new(None),
            children: RefCell::new(Vec::new()),
            components: RefCell::new(TypeMap::new()),

            start_handlers: RefCell::new(Vec::new()),
            update_handlers: RefCell::new(Vec::new()),
            post_update_handlers: RefCell::new(Vec::new()),

            event_handlers: RefCell::new(HashMap::new()),
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
        let mut handlers = event_handlers.entry(TypeId::of::<E>()).or_insert(Vec::new());
        let handler: Box<Fn(Entity, &E, Entity)> = Box::new(handler);
        handlers.push(unsafe { mem::transmute(handler) });
    }

    pub fn recv_by_type(&self, ty: TypeId, event: *const (), send: Entity) {
        let event_handlers = self.event_handlers.borrow();
        if let Some(handlers) = event_handlers.get(&ty) {
            for handler in handlers.iter() {
                handler(self.entity, event, send);
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

#[derive(Clone)]
pub struct SentEvent {
    sender: Entity,
    receiver: Entity,
    ty: TypeId,
    event: *const (),
    time: Scalar,
}

impl SentEvent {
    pub fn new<E: Event>(sender: Entity, event: E, receiver: Entity, time: Scalar) -> SentEvent {
        SentEvent {
            sender: sender,
            receiver: receiver,
            ty: TypeId::of::<E>(),
            event: Box::into_raw(Box::new(event)) as *const (),
            time: time,
        }
    }
}

impl Drop for SentEvent {
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.event as *mut ()) };
    }
}

pub struct World {
    entities: RefCell<HashMap<Entity, Rc<EntityStorage>>>,
    events: RefCell<Vec<SentEvent>>,
}

impl World {
    pub fn new() -> World {
        World {
            entities: RefCell::new(HashMap::new()),
            events: RefCell::new(Vec::new()),
        }
    }

    pub fn update(&self, dt: Scalar) {
        let mut sent_events = Vec::new();
        {
            let mut events = self.events.borrow_mut();
            *events = events.iter().cloned().filter_map(|mut event| {
                if event.time > dt {
                    event.time -= dt;
                    Some(event)
                } else {
                    sent_events.push(event);
                    None
                }
            }).collect();
        }

        for event in sent_events {
            event.sender.send_by_type(event.ty, event.event, event.receiver);
        }
    }

    pub fn all_entities(&self) -> Vec<Entity> {
        self.entities.borrow().keys().cloned().collect()
    }

    fn insert_entity(&self, entity: Entity, storage: EntityStorage) {
        let mut entities = self.entities.borrow_mut();
        entities.insert(entity, Rc::new(storage));
    }

    pub fn send_event(&self, event: SentEvent) {
        let mut events = self.events.borrow_mut();
        events.push(event);
    }

    fn get_entity(&self, entity: &Entity) -> Option<Rc<EntityStorage>> {
        let entities = self.entities.borrow();
        entities.get(entity).cloned()
    }

    pub fn disable_entity(&self, entity: Entity) {
        self.entities.borrow().get(&entity).map(|storage| storage.disabled.set(true));
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
