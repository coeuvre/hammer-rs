use std::collections::HashMap;
use std::marker::PhantomData;
use std::cell::{Ref, RefMut, RefCell};
use std::rc::Rc;

use typemap::{TypeMap, Key};

use Error;

use math::*;

use util::counter::Counter;

use self::component::{Component, ComponentRef};
use self::system::behaviour::{Behaviour, BehaviourDelegate};

pub mod component;
pub mod system;

thread_local!(static WORLD: World = World::new());

thread_local!(static COUNTER: Counter<usize> = Counter::new(0));

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct Entity(usize);

impl Entity {
    pub fn new<S: Into<String>>(id: S) -> Entity {
        let entity = Entity(COUNTER.with(|counter| counter.next()));
        let storage = EntityStorage::new(entity, id.into());
        WORLD.with(|world| world.insert_entity(entity, storage));
        entity
    }

    pub fn as_ref(&self) -> Option<EntityRef> {
        WORLD.with(|world| world.entity_ref(self))
    }

    pub fn add_component<C: Component>(&self, component: C) {
        if let Some(entity) = self.as_ref() {
            entity.add_component(component);
        }
    }

    pub fn add_behaviour<B: Behaviour + 'static>(&self, behaviour: B) {
        self.add_component(BehaviourDelegate::new(behaviour));
    }

    pub fn add_child(&self, child: Entity) -> Result<(), Error> {
        if let Some(this) = self.as_ref() {
            this.add_child(child)
        } else {
            Err("Entity does not exists.".into())
        }
    }

    pub fn id(&self) -> String {
        self.as_ref().map(|entity| entity.id()).unwrap_or("".to_string())
    }

    pub fn parent(&self) -> Option<Entity> {
        self.as_ref().and_then(|entity| entity.parent())
    }

    pub fn component<C: Component>(&self) -> Option<ComponentRef<C>> {
        self.as_ref().and_then(|entity| entity.component::<C>())
    }

    pub fn transform_to_world(&self) -> Transform {
        self.as_ref().map(|entity| entity.transform_to_world()).unwrap_or(Transform::identity())
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

    pub fn add_child(&self, child: Entity) -> Result<(), Error> {
        self.write().add_child(child)
    }

    pub fn component<C: Component>(&self) -> Option<ComponentRef<C>> {
        self.read().component::<C>()
    }

    pub fn id(&self) -> String {
        self.read().id().to_string()
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

    pub fn add_child(&mut self, entity: Entity) -> Result<(), Error> {
        if let Some(entity_ref) = entity.as_ref() {
            let child_id = entity.id();
            for child in self.children() {
                if child.id() == child_id {
                    return Err("Entity with id {} already exists.".into());
                }
            }
            entity_ref.write().parent = Some(self.entity);
            self.children.push(entity);
            Ok(())
        } else {
            Err("Entity does not exists.".into())
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
