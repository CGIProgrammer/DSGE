use crate::components::*;
pub use crate::components::visual::*;
pub use crate::components::camera::*;

use crate::types::*;
use crate::shader::{ShaderStructUniform};
use crate::texture::Texture;
use crate::references::*;
use bytemuck::{Zeroable, Pod};
use nalgebra::Transform3;

pub type GameObjectRef = RcBox<GameObject>;
pub type Transform = Mat4;

#[derive(Clone)]
pub struct GOTransform
{
    pub local : Transform,
    pub global : Transform,
    pub global_for_render: Transform,
    pub global_for_render_prev: Transform,
    _owner : Option<GameObjectRef>,
    _parent : Option<GameObjectRef>,
    _children : Vec::<GameObjectRef>
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
            global_for_render : Transform::identity(),
            global_for_render_prev : Transform::identity(),
            _parent: None,
            _children: Vec::new(),
            _owner: None
        }
    }

    pub fn uniform_value(&self) -> GOTransformUniform
    {
        GOTransformUniform {
            transform: self.global_for_render.as_slice().try_into().unwrap(),
            transform_prev: self.global_for_render_prev.as_slice().try_into().unwrap()
        }
    }
}


#[derive(Clone)]
pub struct GameObject
{
    transform: GOTransform,
    name: String,
    camera: Option<CameraComponent>,
    mesh_visual: Option<MeshVisual>,
    light: Option<Light>,
    components: Vec<RcBox<dyn Component>>
}

#[allow(dead_code)]
impl GameObject
{
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
    
    pub fn new<T: ToString>(name: T) -> RcBox<Self>
    {
        let obj = RcBox::construct(Self {
            name: name.to_string(),
            transform: GOTransform::identity(),
            camera: None,
            mesh_visual: None,
            light: None,
            components: Vec::new()
        });
        obj.take_mut().transform._owner = Some(obj.clone());
        obj
    }

    pub fn set_parent(&mut self, parent: RcBox<GameObject>)
    {
        self.remove_parent();
        let owner = self.transform._owner.clone().unwrap();
        let owner_id = owner.box_id();
        let mut _par = Some(parent.clone());
        loop {
            match _par {
                Some(par) => {
                    let parent_id = par.box_id();
                    if parent_id==owner_id {
                        panic!("Обнаружена циклическая зависимость объектов.");
                    }
                    _par = par.take().transform()._parent.clone();
                },
                None => {
                    break;
                }
            }
        };
        self.transform._parent = Some(parent.clone());
        parent.take().transform_mut()._children.push(self.transform._owner.clone().unwrap());
    }

    pub fn add_child(&mut self, child: RcBox<GameObject>)
    {
        child.take().transform_mut()._parent = self.transform._owner.clone();
        self.transform._children.push(child.clone());
    }

    pub fn add_component<T: Component>(&mut self, component: T)
    {
        let cmp_dyn = (&component) as &dyn std::any::Any;
        if cmp_dyn.is::<CameraComponent>() {
            self.camera = Some(cmp_dyn.downcast_ref::<CameraComponent>().unwrap().clone());
            return;
        }
        if cmp_dyn.is::<MeshVisual>() {
            self.mesh_visual = Some(cmp_dyn.downcast_ref::<MeshVisual>().unwrap().clone());
            return;
        }
        if cmp_dyn.is::<Light>() {
            self.light = Some(cmp_dyn.downcast_ref::<Light>().unwrap().clone());
            return;
        }
        self.components.push(RcBox::construct(component));
    }

    pub fn get_component<T: Component>(&self) -> Option<&RcBox<(dyn Component)>>
    {
        self.components.iter().find(|b| {
            (*b).lock().unwrap().as_any().downcast_ref::<T>().is_some()
        })
    }

    pub fn get_components<T: Component>(&self) -> Vec<&RcBox<(dyn Component)>>
    {
        self.components.iter().filter(|b| {
            (*b).lock().unwrap().as_any().downcast_ref::<T>().is_some()
        }).collect::<Vec<_>>()
    }

    pub fn get_all_components(&self) -> &Vec<RcBox<(dyn Component)>>
    {
        &self.components
    }

    pub fn remove_parent(&mut self)
    {
        if self.transform._parent.is_none() {
            return;
        }
        let _parent = self.transform._parent.clone().unwrap();
        let mut parent = _parent.take();
        let owner = self.transform._owner.clone().unwrap();
        let owner_id : usize = owner.box_id() as _;
        let mut index = usize::MAX;
        for (i, child) in parent.transform()._children.iter().enumerate() {
            let child_id : usize = child.box_id() as _;
            if child_id == owner_id {
                index = i;
                break;
            }
        }
        if index < usize::MAX {
            parent.transform_mut()._children.remove(index);
        }
        self.transform._parent = None;
    }

    pub fn children(&self) -> Vec<RcBox<GameObject>>
    {
        return self.transform._children.clone();
    }

    pub fn transform(&self) -> &GOTransform
    {
        &self.transform
    }
    
    pub fn transform_mut(&mut self) -> &mut GOTransform
    {
        &mut self.transform
    }
    
    pub fn apply_transform(&mut self)
    {
        match &self.transform._parent {
            Some(parent) => {
                let par = parent.take();
                self.transform.global = par.transform().global * self.transform.local;
            },
            None => {
                self.transform.global = self.transform.local;
            }
        };
        fn apply_transform_closure(obj: &mut GameObject) {
            for child in obj.children() {
                let mut ch = child.take();
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
            Some(parent) => {
                let par = parent.take();
                self.transform.global = par.transform().global * self.transform.local;
            },
            None => {
                self.transform.global = self.transform.local;
            }
        };
        fn next_frame_closure(obj: &mut GameObject) {
            let mut transform = obj.transform_mut();
            transform.global_for_render_prev = transform.global_for_render;
            transform.global_for_render = transform.global;
            for child in obj.children() {
                let mut ch = child.take();
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
