use std::collections::HashMap;
use std::marker::PhantomData;
use std::cell::{Ref, RefMut, RefCell};
use std::rc::Rc;

use typemap::{TypeMap, Key};

use math::*;
use renderer::*;
use scene::*;

use util::counter::Counter;

use self::component::{Component, ComponentRef};
use self::component::sprite::Sprite;

pub mod component;

thread_local!(static WORLD: World = World::new());

thread_local!(static COUNTER: Counter<usize> = Counter::new());

pub trait System {
    fn process(&mut self, scene: &mut Scene);
}

pub struct RenderSystem {
    renderer: Renderer,
}

impl RenderSystem {
    pub fn new(renderer: Renderer) -> RenderSystem {
        RenderSystem {
            renderer: renderer,
        }
    }

    fn process_entity(&mut self, entity: Entity) {
        if let Some(entity) = entity.get_ref() {
            let entity = entity.read();

            let mut orig_trans = None;

            if let Some(trans) = entity.component::<Transform>() {
                orig_trans = Some(*self.renderer.trans());
                self.renderer.set_trans(*trans.read() * orig_trans.unwrap());
            }

            if let Some(sprite) = entity.component::<Sprite>() {
                let sprite = sprite.read();
                if let Some(image) = sprite.image() {
                    self.renderer.rect(Rect::with_min_size(vec2(0.0, 0.0), sprite.region().size())).texture(image).draw();
                }
            }

            for child in entity.children() {
                self.process_entity(*child);
            }

            if let Some(orig_trans) = orig_trans {
                self.renderer.set_trans(orig_trans);
            }
        }
    }
}

impl System for RenderSystem {
    fn process(&mut self, scene: &mut Scene) {
        let view_w = 640.0;
        let view_h = 480.0;

        self.renderer.ortho(0.0, view_w, 0.0, view_h);

        self.renderer.clear(0.0, 0.0, 0.0, 1.0);

        self.process_entity(scene.root());

        self.renderer.present();
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
        let storage = EntityStorage::new(id);
        WORLD.with(|world| world.insert_entity(entity, storage));
        entity
    }

    pub fn get_ref(&self) -> Option<EntityRef> {
        WORLD.with(|world| world.entity_ref(self))
    }

    pub fn add_component<C: Component>(&self, component: C) {
        if let Some(entity) = self.get_ref() {
            entity.write().add_component(component);
        }
    }

    pub fn add_child(&mut self, child: Entity) {
        if let Some(parent) = self.get_ref() {
            parent.write().add_child(child);
        }
    }

    pub fn component<C: Component>(&self) -> Option<ComponentRef<C>> {
        self.get_ref().and_then(|entity| entity.read().component::<C>())
    }
}

pub struct EntityStorage {
    id: String,
    parent: Option<Entity>,
    children: Vec<Entity>,
    components: TypeMap,
}

impl EntityStorage {
    pub fn new(id: String) -> EntityStorage {
        EntityStorage {
            id: id,
            parent: None,
            children: Vec::new(),
            components: TypeMap::new(),
        }
    }

    pub fn add_child(&mut self, entity: Entity) {
        self.children.push(entity);
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
