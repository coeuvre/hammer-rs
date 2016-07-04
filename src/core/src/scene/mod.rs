use ecs::*;

pub struct Scene {
    id: String,
    root: Entity,
}

impl Scene {
    pub fn new(id: String) -> Scene {
        Scene {
            id: id,
            root: Entity::new(),
        }
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn root(&self) -> Entity {
        self.root
    }

    pub fn add_entity(&mut self, entity: Entity) {
        if let Some(root) = self.root.get_ref() {
            root.write().add_child(entity);
        }
    }
}
