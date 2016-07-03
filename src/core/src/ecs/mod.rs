use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use typemap::{TypeMap, Key};

use asset::*;
use math::*;
use renderer::*;
use scene::*;

use util::counter::Counter;

lazy_static! {
    static ref COUNTER: Counter<usize> = Counter::new();
    static ref WORLD: World = World::new();
}

pub trait Component: Any + Clone + Default {}

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
        let mut orig_trans = None;

        let entity = entity.get_ref();
        let entity = entity.read();

        if let Some(trans) = entity.get_component::<Transform>() {
            orig_trans = Some(*self.renderer.trans());
            self.renderer.set_trans(*trans.read() * orig_trans.unwrap());
        }

        if let Some(sprite) = entity.get_component::<Sprite>() {
            let sprite = sprite.read();
            if let Some(ref image) = sprite.image {
                self.renderer.rect(Rect::with_min_size(vec2(0.0, 0.0), sprite.region.size())).texture(image).draw();
            }
        }

        for child in entity.get_children() {
            self.process_entity(*child);
        }

        if let Some(orig_trans) = orig_trans {
            self.renderer.set_trans(orig_trans);
        }
    }
}

impl System for RenderSystem {
    fn process(&mut self, scene: &mut Scene) {
        let view_w = 640.0;
        let view_h = 480.0;

        self.renderer.ortho(0.0, view_w, 0.0, view_h);

        self.renderer.clear(0.0, 0.0, 0.0, 1.0);

        self.process_entity(scene.get_root());

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
        let entity = Entity(COUNTER.next());
        let storage = EntityStorage::new(id);
        WORLD.insert_entity(entity, storage);
        entity
    }

    fn get_ref(&self) -> EntityRef {
        WORLD.get_entity_ref(self).unwrap()
    }

    pub fn add_child(&mut self, child: Entity) {
        if let Some(entity) = WORLD.get_entity_ref(self) {
            entity.write().add_child(child);
        }
    }

    pub fn add_component<C: Component>(&mut self, component: C) {
        if let Some(entity) = WORLD.get_entity_ref(self) {
            entity.write().add_component(component);
        }
    }

    pub fn get_parent(&self) -> Option<Entity> {
        WORLD.get_entity_ref(self).and_then(|entity| entity.read().get_parent())
    }

    pub fn get_component<C: Component>(&self) -> Option<ComponentRef<C>> {
        WORLD.get_entity_ref(self).and_then(|entity| entity.read().get_component())
    }
}

pub struct ComponentRef<C: Component> {
    component: Arc<RwLock<C>>,
}

impl<C: Component> ComponentRef<C> {
    pub fn new(component: C) -> ComponentRef<C> {
        ComponentRef {
            component: Arc::new(RwLock::new(component)),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<C> {
        self.component.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<C> {
        self.component.write().unwrap()
    }
}

impl<C: Component> Clone for ComponentRef<C> {
    fn clone(&self) -> ComponentRef<C> {
        ComponentRef {
            component: self.component.clone(),
        }
    }
}

struct EntityStorage {
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

    pub fn get_id(&self) -> &str {
        self.id.as_str()
    }

    pub fn get_parent(&self) -> Option<Entity> {
        self.parent
    }

    pub fn get_children(&self) -> &[Entity] {
        self.children.as_ref()
    }

    pub fn get_component<C: Component>(&self) -> Option<ComponentRef<C>> {
        self.components.get::<ComponentTypeMapKey<C>>().cloned()
    }
}

unsafe impl Sync for EntityStorage {}
unsafe impl Send for EntityStorage {}

struct ComponentTypeMapKey<C: Component> {
    phantom: PhantomData<C>,
}

impl<C: Component> Key for ComponentTypeMapKey<C> {
    type Value = ComponentRef<C>;
}

struct EntityRef {
    entity: Arc<RwLock<EntityStorage>>,
}

impl EntityRef {
    pub fn new(storage: EntityStorage) -> EntityRef {
        EntityRef {
            entity: Arc::new(RwLock::new(storage)),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<EntityStorage> {
        self.entity.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<EntityStorage> {
        self.entity.write().unwrap()
    }
}

impl Clone for EntityRef {
    fn clone(&self) -> EntityRef {
        EntityRef {
            entity: self.entity.clone(),
        }
    }
}

struct World {
    entities: Arc<RwLock<HashMap<Entity, EntityRef>>>,
}

impl World {
    pub fn new() -> World {
        World {
            entities: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert_entity(&self, entity: Entity, storage: EntityStorage) {
        let mut entities = self.entities.write().unwrap();
        entities.insert(entity, EntityRef::new(storage));
    }

    pub fn get_entity_ref(&self, entity: &Entity) -> Option<EntityRef> {
        let entities = self.entities.read().unwrap();
        entities.get(entity).cloned()
    }
}

#[derive(Clone, Default)]
pub struct Sprite {
    image: Option<ImageRef>,
    region: Rect,
    anchor: Vec2,
}

impl Component for Sprite {}

impl Sprite {
    pub fn new(path: &str) -> Sprite {
        let image = asset::<Image>::new().load(path).unwrap();
        Sprite::with_image(image)
    }

    pub fn with_image(image: ImageRef) -> Sprite {
        let (w, h) = image.read().size();
        Sprite {
            image: Some(image),
            region: Rect::with_min_size(vec2(0.0, 0.0), vec2(w as f32, h as f32)),
            anchor: vec2(0.0, 0.0),
        }
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

impl Component for Trans {}
