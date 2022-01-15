use crate::references::*;
use crate::types::*;
use crate::shader::{ShaderStructUniform};
use crate::texture::TextureRef;
use crate::components::Component;

pub type GameObjectRef = RcBox<GameObject>;


#[derive(Copy, Clone)]
pub struct GOTransform
{
    pub local : Mat4,
    pub global : Mat4,
    pub global_for_render: Mat4,
    pub global_for_render_prev: Mat4
}

#[derive(Copy, Clone)]
pub struct GOTransfotmUniform
{
    pub transform : Mat4,
    pub transform_prev : Mat4
}

impl ShaderStructUniform for GOTransfotmUniform
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

#[allow(dead_code)]
impl GOTransform
{
    pub fn identity() -> Self
    {
        Self {
            local : Mat4::identity(),
            global : Mat4::identity(),
            global_for_render : Mat4::identity(),
            global_for_render_prev : Mat4::identity()
        }
    }

    pub fn uniform_value(&self) -> GOTransfotmUniform
    {
        GOTransfotmUniform {
            transform: self.global_for_render,
            transform_prev: self.global_for_render_prev
        }
    }
}

#[allow(dead_code)]
pub struct GameObjectComposite
{
    obj : GameObject,
}

#[allow(dead_code)]
impl GameObjectComposite
{
    pub fn component<C : Component + 'static>(mut self, component : C) -> Self
    {
        self.obj.components.push(Box::new(component));
        self
    }

    pub fn build(self) -> GameObjectRef
    {
        let obj = GameObjectRef::construct(self.obj);
        for comp in &mut obj.take_mut().components {
            comp.set_owner(obj.clone());
        }
        obj
    }
}

#[allow(dead_code)]
pub struct GameObject
{
    name : String,
    pub transform : GOTransform,
    parent : Option<GameObjectRef>,
    children : Vec<GameObjectRef>,
    components : Vec<Box<dyn Component>>
}

#[allow(dead_code)]
impl GameObject
{
    pub fn composite(name : &str) -> GameObjectComposite
    {
        GameObjectComposite {
            obj : Self::empty(name)
        }
    }

    pub fn empty(name : &str) -> Self
    {
        Self {
            name : name.to_string(),
            transform : GOTransform::identity(),
            children : Vec::new(),
            parent : None,
            components : Vec::new()
        }
    }

    pub fn apply_children_transformation(&mut self)
    {
        for _obj in &mut self.children
        {
            let mut obj = _obj.take_mut();
            obj.transform.global = self.transform.global * obj.transform.local;
            obj.apply_children_transformation();
        }
    }
}