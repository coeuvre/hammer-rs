use std::cell::{Ref, RefMut, RefCell};
use std::rc::Rc;

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

#[derive(Clone)]
pub struct SceneRef {
    scene: Rc<RefCell<Scene>>,
}

impl SceneRef {
    fn new(scene: Scene) -> SceneRef {
        SceneRef {
            scene: Rc::new(RefCell::new(scene)),
        }
    }

    pub fn read(&self) -> Ref<Scene> {
        self.scene.borrow()
    }

    pub fn write(&self) -> RefMut<Scene> {
        self.scene.borrow_mut()
    }
}

struct SceneManager {
    scene_stack: RefCell<Vec<SceneRef>>,
}

impl SceneManager {
    pub fn new() -> SceneManager {
        SceneManager {
            scene_stack: RefCell::new(Vec::new()),
        }
    }

    pub fn top(&self) -> Option<SceneRef> {
        let stack = self.scene_stack.borrow();
        stack.last().cloned()
    }

    pub fn push(&self, scene: Scene) {
        let mut stack = self.scene_stack.borrow_mut();
        stack.push(SceneRef::new(scene));
    }

    pub fn pop(&self) {
        let mut stack = self.scene_stack.borrow_mut();
        stack.pop();
    }

    pub fn switch(&self, scene: Scene) {
        let mut stack = self.scene_stack.borrow_mut();
        stack.pop();
        stack.push(SceneRef::new(scene));
    }
}

thread_local!(static SCENE_MANAGER: SceneManager = SceneManager::new());

pub fn top() -> Option<SceneRef> {
    SCENE_MANAGER.with(|scene_manager| scene_manager.top())
}

pub fn push(scene: Scene) {
    SCENE_MANAGER.with(|scene_manager| scene_manager.push(scene));
}

pub fn pop() {
    SCENE_MANAGER.with(|scene_manager| scene_manager.pop());
}

pub fn switch(scene: Scene) {
    SCENE_MANAGER.with(|scene_manager| scene_manager.switch(scene));
}
