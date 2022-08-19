use std::collections::HashMap;

use crate::components::*;
pub use crate::components::visual::*;
pub use crate::components::camera::*;
use crate::game_logic::Behaviour;
use crate::game_logic::behaviour::DynBehaviour;

use crate::types::*;
use crate::shader::{ShaderStructUniform};
use crate::texture::Texture;
use crate::scene::SceneRef;
use crate::references::*;
use bytemuck::{Zeroable, Pod};

pub type GameObjectRef = RcBox<GameObject>;
pub type Transform = Mat4;

#[derive(Clone)]
pub enum GOParent
{
    Object(GameObjectRef),
    Scene(SceneRef),
    None
}

impl GOParent
{
    pub fn is_some(&self) -> bool
    {
        match self
        {
            Self::Object(_) | Self::Scene(_) => true,
            Self::None => false
        }
    }

    pub fn is_none(&self) -> bool
    {
        match self {Self::None => true, _ => false}
    }
}

#[derive(Clone)]
pub struct GOTransform
{
    pub local : Transform,
    pub global : Transform,
    pub global_prev: Transform,
    pub(crate) _owner : Option<GameObjectRef>,
    pub(crate) _parent : GOParent,
    pub(crate) _children : HashMap::<i32, GameObjectRef>
}

#[repr(C)]
#[derive(Copy, Clone, Default, Zeroable, Pod)]
pub struct GOTransformUniform
{
    pub transform : [f32; 16],
    pub transform_prev : [f32; 16]
}

impl ShaderStructUniform for GOTransformUniform
{
    fn glsl_type_name() -> String
    {
        String::from("GOTransform")
    }

    fn structure() -> String
    {
        String::from("{
            mat4 transform;
            mat4 transform_prev;
        }")
    }
    
    fn texture(&self) -> Option<&Texture>
    {
        None
    }
}

#[allow(dead_code)]
impl GOTransform
{

    fn set_owner(&mut self, owner: RcBox<GameObject>)
    {
        self._owner = Some(owner);
    }

    pub fn identity() -> Self
    {
        Self {
            local : Transform::identity(),
            global : Transform::identity(),
            global_prev : Transform::identity(),
            _parent: GOParent::None,
            _children: HashMap::new(),
            _owner: None,
        }
    }

    pub fn uniform_value(&self) -> GOTransformUniform
    {
        GOTransformUniform {
            transform: self.global.as_slice().try_into().unwrap(),
            transform_prev: self.global_prev.as_slice().try_into().unwrap()
        }
    }
}


#[derive(Clone)]
pub struct GameObject
{
    pub(crate) scene: Option<SceneRef>,
    pub(crate) transform: GOTransform,
    name: String,
    camera: Option<CameraComponent>,
    mesh_visual: Option<MeshVisual>,
    light: Option<Light>,
    components: Vec<DynBehaviour>,
    //scene: Option<SceneRef>
}

impl Drop for GameObject {
    fn drop(&mut self) {
        println!("GameObject dropped");
    }
}

#[allow(dead_code)]
impl GameObject
{
    /*pub fn set_scene(&mut self, scene: SceneRef)
    {
        self.scene = Some(scene);
    }*/

    #[inline]
    pub fn name(&self) -> &String
    {
        &self.name
    }

    pub fn camera(&self) -> Option<&CameraComponent>
    {
        self.camera.as_ref()
    }

    pub fn camera_mut(&mut self) -> Option<&mut CameraComponent>
    {
        self.camera.as_mut()
    }
    
    pub fn visual(&self) -> Option<&MeshVisual>
    {
        self.mesh_visual.as_ref()
    }
    
    pub fn light(&self) -> Option<&Light>
    {
        self.light.as_ref()
    }
    
    pub fn light_mut(&mut self) -> Option<&mut Light>
    {
        self.light.as_mut()
    }
    
    pub fn new<T: ToString>(name: T) -> RcBox<Self>
    {
        let obj = RcBox::construct(Self {
            name: name.to_string(),
            transform: GOTransform::identity(),
            camera: None,
            mesh_visual: None,
            light: None,
            components: Vec::new(),
            scene: None
        });
        obj.lock_write().transform._owner = Some(obj.clone());
        obj
    }

    pub fn parent_object(&self) -> Option<&GameObjectRef>
    {
        if let GOParent::Object(ref parent) = self.transform._parent {
            Some(parent)
        }
        else {
            None
        }
    }

    pub fn is_root(&self) -> bool 
    {
        if let GOParent::Scene(ref _s) = self.transform._parent {
            return true;
        } else {
            return false;
        }
    }

    pub fn set_parent(&mut self, parent: RcBox<GameObject>)
    {
        self.remove_parent();
        let mut _par = GOParent::Object(parent.clone());
        let parent_id = parent.box_id();
        loop {
            match _par {
                GOParent::Object(ref par) => {
                    if parent_id == par.box_id() {
                        panic!("Обнаружена циклическая зависимость объектов.");
                    }
                },
                _ => {
                    break;
                }
            }
        };
        self.transform._parent = GOParent::Object(parent.clone());
        let owner = self.transform._owner.clone().unwrap();
        parent.lock().transform_mut()._children.insert(owner.box_id(), owner);
    }

    pub fn add_child(&mut self, child: RcBox<GameObject>)
    {
        child.lock().set_parent(self.transform._owner.clone().unwrap());
    }

    /// Добавляет компонент и возвращает RcBox с этим компонентом
    pub fn add_component<T: Behaviour>(&mut self, component: T) -> Option<RcBox<T>>
    {
        let cmp_dyn = (&component) as &dyn std::any::Any;
        if cmp_dyn.is::<CameraComponent>() {
            self.camera = Some(cmp_dyn.downcast_ref::<CameraComponent>().unwrap().clone());
            return None;
        }
        if cmp_dyn.is::<MeshVisual>() {
            self.mesh_visual = Some(cmp_dyn.downcast_ref::<MeshVisual>().unwrap().clone());
            return None;
        }
        if cmp_dyn.is::<Light>() {
            self.light = Some(cmp_dyn.downcast_ref::<Light>().unwrap().clone());
            return None;
        }
        let result = RcBox::construct(component);
        self.components.push(result.clone());
        if let Some(ref scene) = self.scene {
            scene.lock().event_processor.update_object(self);
        };
        Some(result)
    }

    pub fn get_component<T: Behaviour>(&self) -> Option<&DynBehaviour>
    {
        self.components.iter().find(|b| {
            (*b).lock().unwrap().as_any().downcast_ref::<T>().is_some()
        })
    }

    pub fn get_components<T: Behaviour>(&self) -> Vec<&DynBehaviour>
    {
        self.components.iter().filter(|b| {
            (*b).lock().unwrap().as_any().downcast_ref::<T>().is_some()
        }).collect::<Vec<_>>()
    }

    pub fn get_all_components(&self) -> &Vec<DynBehaviour>
    {
        &self.components
    }

    pub fn remove_parent(&mut self)
    {
        if self.transform._parent.is_none() {
            return;
        }
        match self.transform._parent {
            GOParent::Scene(ref scene) => {
                scene.lock_write().root_objects.remove(&(self as *const Self as usize as i32));
            },
            GOParent::Object(ref object) => {
                object.lock_write().transform._children.remove(&(self as *const Self as usize as i32));
            },
            GOParent::None => ()
        }
        match self.scene {
            Some(ref scene) => 
                self.transform._parent = GOParent::Scene(scene.clone()),
            None => 
                self.transform._parent = GOParent::None,
        };
    }

    pub fn children(&self) -> Vec<RcBox<GameObject>>
    {
        self.transform._children.iter().map(|(_,v)| v.clone()).collect()
    }

    pub fn transform(&self) -> &GOTransform
    {
        &self.transform
    }
    
    pub fn transform_mut(&mut self) -> &mut GOTransform
    {
        &mut self.transform
    }
    
    pub(crate) fn step(&mut self)
    {
        match &self.transform._parent {
            GOParent::Object(parent) => {
                let par = parent.lock();
                self.transform.global = par.transform().global * self.transform.local;
            },
            _ => {
                self.transform.global = self.transform.local;
            }
        };
        fn apply_transform_closure(obj: &mut GameObject) {
            for child in obj.children() {
                let mut ch = child.lock();
                {
                    let ch_transform = ch.transform_mut();
                    ch_transform.global = obj.transform().global * ch_transform.local;
                }
                apply_transform_closure(&mut *ch);
            }
        }
        apply_transform_closure(self);
    }

    pub fn next_frame(&mut self)
    {
        match &self.transform._parent {
            GOParent::Object(parent) => {
                let par = parent.lock();
                self.transform.global = par.transform().global * self.transform.local;
            },
            _ => {
                self.transform.global = self.transform.local;
            }
        };
        fn next_frame_closure(obj: &mut GameObject) {
            let mut transform = obj.transform_mut();
            transform.global_prev = transform.global;
            transform.global = transform.global;
            for child in obj.children() {
                let mut ch = child.lock();
                {
                    let ch_transform = ch.transform_mut();
                    ch_transform.global = obj.transform().global * ch_transform.local;
                }
                next_frame_closure(&mut *ch);
            }
        }
        next_frame_closure(self);
    }

    pub fn fork(&self) -> RcBox<GameObject>
    {
        RcBox::construct(self.clone())
    }
}
