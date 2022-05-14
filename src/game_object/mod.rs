mod mesh_object;
mod camera_object;

pub use mesh_object::*;
pub use camera_object::*;

use crate::types::*;
use crate::shader::{ShaderStructUniform};
use crate::texture::TextureRef;
use crate::references::*;
use bytemuck::{Zeroable, Pod};

#[derive(Clone)]
pub struct GOTransform
{
    pub local : Mat4,
    pub global : Mat4,
    pub global_for_render: Mat4,
    pub global_for_render_prev: Mat4,
    _owner : Option<RcBox<dyn GameObject>>,
    _parent : Option<RcBox<dyn GameObject>>,
    _children : Vec::<RcBox<dyn GameObject>>
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
    
    fn texture(&self) -> Option<&TextureRef>
    {
        None
    }
}

impl GOTransform
{
    fn set_owner(&mut self, owner: RcBox<dyn GameObject>)
    {
        self._owner = Some(owner);
    }

    pub fn identity() -> Self
    {
        Self {
            local : Mat4::identity(),
            global : Mat4::identity(),
            global_for_render : Mat4::identity(),
            global_for_render_prev : Mat4::identity(),
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

pub trait GameObject: Send + Sync
{
    fn apply_transform(&mut self);
    fn transform(&self) -> &GOTransform;
    fn transform_mut(&mut self) -> &mut GOTransform;
    fn next_frame(&mut self);
    fn visual(&self) -> Option<&dyn super::AbstractVisualObject>;
    fn visual_mut(&mut self) -> Option<&mut dyn super::AbstractVisualObject>;
    fn camera(&self) -> Option<&dyn super::AbstractCameraObject>;
    fn camera_mut(&mut self) -> Option<&mut dyn super::AbstractCameraObject>;
    fn set_parent(&mut self, parent: RcBox<dyn GameObject>);
    fn add_child(&mut self, child: RcBox<dyn GameObject>);
    fn remove_parent(&mut self);
    fn children(&self) -> Vec<RcBox<dyn GameObject>>;
    fn fork(&self) -> RcBox<dyn GameObject>;
    //fn set_relation(child: RcBox<dyn GameObject>, parent: RcBox<dyn GameObject>);
}

macro_rules! impl_gameobject
{
    ($type_name:ident) => {
        fn set_parent(&mut self, parent: RcBox<dyn GameObject>)
        {
            self.remove_parent();
            let owner = self.transform._owner.clone().unwrap();
            let owner_id = owner.as_ref() as *const std::sync::Mutex<dyn GameObject> as *const i32 as i32;
            let mut _par = Some(parent.clone());
            loop {
                match _par {
                    Some(par) => {
                        let parent_id = par.as_ref() as *const std::sync::Mutex<dyn GameObject> as *const i32 as i32;
                        if parent_id==owner_id {
                            panic!("Обнаружена циклическая зависимость объектов.");
                        }
                        _par = par.lock().unwrap().transform()._parent.clone();
                    },
                    None => {
                        break;
                    }
                }
            };
            self.transform._parent = Some(parent.clone());
            parent.lock().unwrap().transform_mut()._children.push(self.transform._owner.clone().unwrap());
        }

        fn add_child(&mut self, child: RcBox<dyn GameObject>)
        {
            child.lock().unwrap().transform_mut()._parent = self.transform._owner.clone();
            self.transform._children.push(child.clone());
        }

        fn remove_parent(&mut self)
        {
            if self.transform._parent.is_none() {
                return;
            }
            let _parent = self.transform._parent.clone().unwrap();
            let mut parent = _parent.lock().unwrap();
            let owner = self.transform._owner.clone().unwrap();
            let owner_id : usize = owner.as_ref() as *const std::sync::Mutex<dyn GameObject> as *const usize as usize;
            let mut index = usize::MAX;
            for (i, child) in parent.transform()._children.iter().enumerate() {
                let child_id : usize = child.as_ref() as *const std::sync::Mutex<dyn GameObject> as *const usize as usize;
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

        fn children(&self) -> Vec<RcBox<dyn GameObject>>
        {
            return self.transform._children.clone();
        }

        fn transform(&self) -> &GOTransform
        {
            &self.transform
        }
        
        fn transform_mut(&mut self) -> &mut GOTransform
        {
            &mut self.transform
        }
        
        fn apply_transform(&mut self)
        {
            match &self.transform._parent {
                Some(parent) => {
                    let par = parent.lock().unwrap();
                    self.transform.global = par.transform().global * self.transform.local;
                },
                None => {
                    self.transform.global = self.transform.local;
                }
            };
            fn apply_transform_closure(obj: &mut dyn GameObject) {
                for child in obj.children() {
                    let mut ch = child.lock().unwrap();
                    {
                        let ch_transform = ch.transform_mut();
                        ch_transform.global = obj.transform().global * ch_transform.local;
                    }
                    apply_transform_closure(&mut *ch);
                }
            }
            apply_transform_closure(self);
        }

        fn next_frame(&mut self)
        {
            match &self.transform._parent {
                Some(parent) => {
                    let par = parent.lock().unwrap();
                    self.transform.global = par.transform().global * self.transform.local;
                },
                None => {
                    self.transform.global = self.transform.local;
                }
            };
            fn next_frame_closure(obj: &mut dyn GameObject) {
                let mut transform = obj.transform_mut();
                transform.global_for_render_prev = transform.global_for_render;
                transform.global_for_render = transform.global;
                for child in obj.children() {
                    let mut ch = child.lock().unwrap();
                    {
                        let ch_transform = ch.transform_mut();
                        ch_transform.global = obj.transform().global * ch_transform.local;
                    }
                    next_frame_closure(&mut *ch);
                }
            }
            next_frame_closure(self);
        }

        fn fork(&self) -> RcBox<dyn GameObject>
        {
            self.fork_inner()
        }
    };
}

pub(crate) use impl_gameobject;
    