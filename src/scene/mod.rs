use std::collections::HashMap;

use crate::{
    game_logic::events::EventProcessor,
    game_object::{GOParent, GameObjectRef},
    references::{MutexLockBox, RcBox},
    resource_manager::ResourceManager,
    time::UniformTime,
};

use self::scene_loader::read_scene;
pub type SceneRef = RcBox<Scene>;
mod scene_loader;
pub struct Scene {
    pub(crate) root_objects: HashMap<i32, GameObjectRef>,
    pub(crate) event_processor: EventProcessor,
    instance: Option<SceneRef>,
}

impl Scene {
    pub fn new() -> SceneRef {
        let scene = Self {
            root_objects: HashMap::new(),
            event_processor: Default::default(),
            instance: None,
        };
        let instance = RcBox::construct(scene);
        let mut ins = instance.lock();
        ins.instance = Some(instance.clone());
        drop(ins);
        instance
    }

    pub fn time(&self) -> UniformTime {
        *self.event_processor.time.lock()
    }

    pub fn event_processor(&self) -> &EventProcessor {
        &self.event_processor
    }

    pub fn from_file(
        filename: &str,
        resource_manager: &mut ResourceManager,
    ) -> (SceneRef, Option<GameObjectRef>) {
        let scene = Scene::new();
        let (objects, camera) = read_scene(filename, resource_manager);
        let mut _scene = scene.lock();
        for obj in objects {
            _scene.add_object(obj).unwrap();
        }
        drop(_scene);
        (scene, camera)
    }

    pub fn add_object(&mut self, object: GameObjectRef) -> Result<(), String> {
        self.event_processor.update_object(&*object.lock());
        let mut obj = object.lock();
        match obj.scene {
            Some(ref scene) => {
                if self.ref_id() != scene.box_id() {
                    return Err(
                        "Нельзя добавлять объект с другой сцены. Может когда-нибудь разрешу."
                            .to_owned(),
                    );
                };
            }
            None => {
                obj.scene = self.instance.clone();
            }
        };
        match obj.transform._parent {
            GOParent::None => {
                obj.transform._parent = GOParent::Scene(self.instance.as_ref().unwrap().clone());
            }
            _ => (),
        };
        drop(obj);
        self.root_objects.insert(object.box_id(), object);
        Ok(())
    }

    pub fn unlink_object(&mut self, obj: GameObjectRef) {
        let mut object = obj.lock_write();
        self.event_processor.remove_object(obj.clone());
        object.scene = None;
        self.root_objects.remove(&obj.box_id());
    }

    pub fn step(&mut self) {
        for (_, obj) in &self.root_objects {
            let mut _obj = obj.lock();
            _obj.step();
        }
        self.event_processor.step();
    }

    pub fn root_objects(&self) -> Vec<GameObjectRef> {
        self.root_objects.values().map(|obj| obj.clone()).collect()
    }

    pub(crate) fn ref_id(&self) -> i32 {
        self as *const Self as i32
    }
}
