use crate::types::Mat4;
use super::{GOTransform, GameObject};
use crate::references::*;

pub use crate::components::{
    visual::AbstractVisual,
    AbstractCameraObject,
    AbstractCamera,
    camera::CameraUniformData
};

pub struct CameraObject
{
    transform : GOTransform,
    projection : Mat4,
    pub parent : Option<RcBox<dyn GameObject>>,
    pub children : Vec::<RcBox<dyn GameObject>>
}

impl CameraObject
{
    pub fn new(aspect: f32, fov: f32, znear: f32, zfar: f32) -> Self
    {
        Self {
            transform : GOTransform::identity(),
            projection : nalgebra::Perspective3::new(aspect, fov, znear, zfar).as_matrix().clone(),
            parent : None,
            children : Vec::new(),
        }
    }
}

impl AbstractCameraObject for CameraObject {}

impl GameObject for CameraObject
{
    fn transform(&self) -> &GOTransform
    {
        &self.transform
    }

    fn apply_transform(&mut self)
    {
        let transform = self.transform_mut().global;
        for _child in &mut self.children {
            let mut child = _child.lock().unwrap();
            let mut child_transform = child.transform_mut();
            child_transform.global = transform * child_transform.local;
            drop(child_transform);
            child.apply_transform();
        }
    }

    fn transform_mut(&mut self) -> &mut GOTransform
    {
        &mut self.transform
    }
}

impl AbstractCamera for CameraObject
{
    fn projection(&self) -> Mat4
    {
        self.projection
    }
    
    fn set_projection(&mut self, aspect: f32, fov: f32, znear: f32, zfar: f32)
    {
        self.projection = nalgebra::Perspective3::new(aspect, fov, znear, zfar).as_matrix().clone();
    }

    fn uniform_data(&self) -> CameraUniformData
    {
        CameraUniformData {
            transform : self.transform.global_for_render,
            transform_prev : self.transform.global_for_render_prev,
            transform_inverted : self.transform.global_for_render.try_inverse().unwrap(),
            transform_prev_inverted : self.transform.global_for_render_prev.try_inverse().unwrap(),
            projection : self.projection,
            projection_inverted : self.projection.try_inverse().unwrap(),
        }
    }
}