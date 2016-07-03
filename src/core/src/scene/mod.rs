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

    pub fn get_root(&self) -> Entity {
        self.root
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.root.add_child(entity);
    }
}
