use std::any::Any;

use crate::{
    types::Mat4,
};
use super::{GameObject, Component, ProjectionUniformData};

#[derive(Clone)]
pub struct CameraComponent
{
    projection : Mat4,
}

impl CameraComponent
{
    pub fn new(aspect: f32, fov: f32, znear: f32, zfar: f32) -> Self
    {
        Self {
            projection : nalgebra::Perspective3::new(aspect, fov, znear, zfar).as_matrix().clone(),
        }
    }
}

#[allow(dead_code)]
impl CameraComponent
{
    pub fn projection(&self) -> Mat4
    {
        self.projection
    }
    
    pub fn set_projection(&mut self, aspect: f32, fov: f32, znear: f32, zfar: f32)
    {
        self.projection = nalgebra::Perspective3::new(aspect, fov, znear, zfar).as_matrix().clone();
    }

    pub fn uniform_data(&self, obj: &GameObject) -> ProjectionUniformData
    {
        let transform = obj.transform();
        ProjectionUniformData {
            transform : transform.global_for_render.as_slice().try_into().unwrap(),
            transform_prev : transform.global_for_render_prev.as_slice().try_into().unwrap(),
            transform_inverted : transform.global_for_render.try_inverse().unwrap().as_slice().try_into().unwrap(),
            transform_prev_inverted : transform.global_for_render_prev.try_inverse().unwrap().as_slice().try_into().unwrap(),
            projection : self.projection.as_slice().try_into().unwrap(),
            projection_inverted : self.projection.try_inverse().unwrap().as_slice().try_into().unwrap(),
        }
    }
}

impl Component for CameraComponent
{
    fn as_any(&self) -> &dyn Any
    {
        self
    }

    fn on_geometry_pass_init(&mut self, _owner: &GameObject, _renderer: &mut crate::renderer::Renderer) -> Result<ProjectionUniformData, ()>
    {
        Ok(self.uniform_data(_owner))
    }
}