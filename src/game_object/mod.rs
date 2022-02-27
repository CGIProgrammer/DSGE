mod mesh_object;
mod camera_object;

pub use mesh_object::*;
pub use camera_object::*;

use crate::types::*;
use crate::shader::{ShaderStructUniform};
use crate::texture::TextureRef;

#[derive(Copy, Clone)]
pub struct GOTransform
{
    pub local : Mat4,
    pub global : Mat4,
    pub global_for_render: Mat4,
    pub global_for_render_prev: Mat4
}

#[derive(Copy, Clone, Default)]
pub struct GOTransformUniform
{
    pub transform : Mat4,
    pub transform_prev : Mat4
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
    pub fn identity() -> Self
    {
        Self {
            local : Mat4::identity(),
            global : Mat4::identity(),
            global_for_render : Mat4::identity(),
            global_for_render_prev : Mat4::identity()
        }
    }

    pub fn uniform_value(&self) -> GOTransformUniform
    {
        GOTransformUniform {
            transform: self.global_for_render,
            transform_prev: self.global_for_render_prev
        }
    }
}

pub trait GameObject
{
    fn apply_transform(&mut self);
    fn transform(&self) -> &GOTransform;
    fn transform_mut(&mut self) -> &mut GOTransform;
}