use std::any::Any;

use crate::{
    types::Mat4,
};
use super::{GameObject, Component, ProjectionUniformData};

#[derive(Clone)]
pub struct CameraComponent
{
    projection : nalgebra::Perspective3<f32>
}

impl CameraComponent
{
    pub fn new(aspect: f32, fov: f32, znear: f32, zfar: f32) -> Self
    {
        Self {
            projection : nalgebra::Perspective3::new(aspect, fov, znear, zfar),
        }
    }
}

#[allow(dead_code)]
impl CameraComponent
{
    pub fn projection(&self) -> Mat4
    {
        self.projection.as_matrix().clone()
    }
    
    pub fn set_aspect_ratio(&mut self, aspect: f32)
    {
        self.projection.set_aspect(aspect);
    }
    
    pub fn set_aspect_dimenstions(&mut self, width: u16, height: u16)
    {
        self.projection.set_aspect(width as f32 / height as f32);
    }

    pub fn uniform_data(&self, obj: &GameObject) -> ProjectionUniformData
    {
        let transform = obj.transform();
        let projection = self.projection.as_matrix();
        ProjectionUniformData {
            transform : transform.global_for_render.as_slice().try_into().unwrap(),
            transform_prev : transform.global_for_render_prev.as_slice().try_into().unwrap(),
            transform_inverted : transform.global_for_render.try_inverse().unwrap().as_slice().try_into().unwrap(),
            transform_prev_inverted : transform.global_for_render_prev.try_inverse().unwrap().as_slice().try_into().unwrap(),
            projection : projection.as_slice().try_into().unwrap(),
            projection_inverted : projection.try_inverse().unwrap().as_slice().try_into().unwrap(),
        }
    }
}

impl Component for CameraComponent
{
    fn as_any(&self) -> &dyn Any
    {
        self
    }
}