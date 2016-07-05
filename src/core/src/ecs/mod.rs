use std::collections::HashMap;
use std::marker::PhantomData;
use std::cell::{Ref, RefMut, RefCell};
use std::rc::Rc;

use typemap::{TypeMap, Key};

use math::*;
use renderer::{self, Drawable};

use util::counter::Counter;

use self::component::{Component, ComponentRef, BehaviourDelegate};
use self::component::sprite::Sprite;

pub mod component;

thread_local!(static WORLD: World = World::new());

thread_local!(static COUNTER: Counter<usize> = Counter::new());

pub trait System {
    fn start(&mut self, _entity: &EntityRef) {}
    fn update(&mut self, _entity: &EntityRef) {}
}

pub struct RenderSystem {}

impl System for RenderSystem {
    fn update(&mut self, entity: &EntityRef) {
        if let Some(sprite) = entity.component::<Sprite>() {
            let trans = entity.transform_to_world();
            renderer::set_transform(trans);
            let sprite = sprite.read();
            if let Some(image) = sprite.image() {
                renderer::rect(Rect::with_min_size(vector(0.0, 0.0), sprite.region().size())).texture(image).draw();
            }
        }
    }
}

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct Entity(usize);

impl Entity {
    pub fn new() -> Entity {
        Entity::with_id("Entity".to_string())
    }

    pub fn with_id(id: String) -> Entity {
        let entity = Entity(COUNTER.with(|counter| counter.next()));
        let storage = EntityStorage::new(entity, id);
        WORLD.with(|world| world.insert_entity(entity, storage));
        entity
    }

    pub fn get_ref(&self) -> Option<EntityRef> {
        WORLD.with(|world| world.entity_ref(self))
    }

    pub fn add_component<C: Component>(&self, component: C) {
        if let Some(entity) = self.get_ref() {
            entity.add_component(component);
        }
    }

    pub fn add_child(&self, child: Entity) {
        if let Some(parent) = self.get_ref() {
            parent.add_child(child);
        }
    }

    pub fn parent(&self) -> Option<Entity> {
        self.get_ref().and_then(|entity| entity.parent())
    }

    pub fn component<C: Component>(&self) -> Option<ComponentRef<C>> {
        self.get_ref().and_then(|entity| entity.component::<C>())
    }

    pub fn transform_to_world(&self) -> Transform {
        self.get_ref().map(|entity| entity.transform_to_world()).unwrap_or(Transform::identity())
    }
}

#[derive(Clone)]
pub struct EntityRef {
    storage: Rc<RefCell<EntityStorage>>,
}

impl EntityRef {
    fn new(storage: EntityStorage) -> EntityRef {
        EntityRef {
            storage: Rc::new(RefCell::new(storage)),
        }
    }

    pub fn read(&self) -> Ref<EntityStorage> {
        self.storage.borrow()
    }

    pub fn write(&self) -> RefMut<EntityStorage> {
        self.storage.borrow_mut()
    }

    pub fn add_component<C: Component>(&self, component: C) {
        self.write().add_component(component);
    }

    pub fn add_child(&self, child: Entity) {
        self.write().add_child(child);
    }

    pub fn component<C: Component>(&self) -> Option<ComponentRef<C>> {
        self.read().component::<C>()
    }

    pub fn parent(&self) -> Option<Entity> {
        self.read().parent()
    }

    pub fn children(&self) -> Vec<Entity> {
        self.read().children().to_vec()
    }

    pub fn transform_to_world(&self) -> Transform {
        let transform = self.component::<Transform>().map(|transform| *transform.read()).unwrap_or(Transform::identity());
        let parent = self.parent().map(|parent| parent.transform_to_world());
        parent.map(|parent| transform * parent).unwrap_or(transform)
    }
}

pub struct EntityStorage {
    entity: Entity,
    id: String,
    parent: Option<Entity>,
    children: Vec<Entity>,
    components: TypeMap,
}

impl EntityStorage {
    pub fn new(entity: Entity, id: String) -> EntityStorage {
        EntityStorage {
            entity: entity,
            id: id,
            parent: None,
            children: Vec::new(),
            components: TypeMap::new(),
        }
    }

    pub fn add_child(&mut self, entity: Entity) {
        if let Some(entity_ref) = entity.get_ref() {
            entity_ref.write().parent = Some(self.entity);
            self.children.push(entity);
        }
    }

    pub fn add_component<C: Component>(&mut self, component: C) {
        self.components.insert::<ComponentTypeMapKey<C>>(ComponentRef::new(component));
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn parent(&self) -> Option<Entity> {
        self.parent
    }

    pub fn children(&self) -> &[Entity] {
        self.children.as_ref()
    }

    pub fn component<C: Component>(&self) -> Option<ComponentRef<C>> {
        self.components.get::<ComponentTypeMapKey<C>>().cloned()
    }
}

struct ComponentTypeMapKey<C: Component> {
    phantom: PhantomData<C>,
}

impl<C: Component> Key for ComponentTypeMapKey<C> {
    type Value = ComponentRef<C>;
}

struct World {
    entities: RefCell<HashMap<Entity, EntityRef>>,
}

impl World {
    pub fn new() -> World {
        World {
            entities: RefCell::new(HashMap::new()),
        }
    }

    pub fn insert_entity(&self, entity: Entity, storage: EntityStorage) {
        let mut entities = self.entities.borrow_mut();
        entities.insert(entity, EntityRef::new(storage));
    }

    pub fn entity_ref(&self, entity: &Entity) -> Option<EntityRef> {
        let entities = self.entities.borrow();
        entities.get(entity).cloned()
    }
}

pub struct BehaviourSystem {}

impl System for BehaviourSystem {
    fn start(&mut self, entity: &EntityRef) {
        if let Some(behaviour_delegate) = entity.component::<BehaviourDelegate>() {
            behaviour_delegate.write().start(&entity);
        }
    }

    fn update(&mut self, entity: &EntityRef) {
        if let Some(behaviour_delegate) = entity.component::<BehaviourDelegate>() {
            behaviour_delegate.write().update(&entity);
        }
    }
}
