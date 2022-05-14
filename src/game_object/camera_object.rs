use crate::types::Mat4;
use super::{GOTransform, GameObject};
use crate::references::*;
use super::impl_gameobject;

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
}

impl CameraObject
{
    pub fn new(aspect: f32, fov: f32, znear: f32, zfar: f32) -> RcBox<dyn GameObject>
    {
        let result = Self {
            transform : GOTransform::identity(),
            projection : nalgebra::Perspective3::new(aspect, fov, znear, zfar).as_matrix().clone(),
        };
        let result = RcBox::construct(result);
        result.take_mut().transform.set_owner(result.clone());
        result
    }
	
	fn fork_inner(&self) -> RcBox<dyn GameObject>
	{
		let result = Self {
            transform : self.transform.clone(),
            projection : self.projection,
        };
        let result = RcBox::construct(result);
        result.take_mut().transform.set_owner(result.clone());
        result
	}
}

impl AbstractCameraObject for CameraObject {}

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
            transform : self.transform.global_for_render.as_slice().try_into().unwrap(),
            transform_prev : self.transform.global_for_render_prev.as_slice().try_into().unwrap(),
            transform_inverted : self.transform.global_for_render.try_inverse().unwrap().as_slice().try_into().unwrap(),
            transform_prev_inverted : self.transform.global_for_render_prev.try_inverse().unwrap().as_slice().try_into().unwrap(),
            projection : self.projection.as_slice().try_into().unwrap(),
            projection_inverted : self.projection.try_inverse().unwrap().as_slice().try_into().unwrap(),
        }
    }
}
impl GameObject for CameraObject
{
    fn visual(&self) -> Option<&dyn super::AbstractVisualObject>
    {
        None
    }

    fn visual_mut(&mut self) -> Option<&mut dyn super::AbstractVisualObject>
    {
        None
    }

    fn camera(&self) -> Option<&dyn super::AbstractCameraObject>
    {
        Some(self)
    }

    fn camera_mut(&mut self) -> Option<&mut dyn super::AbstractCameraObject>
    {
        Some(self)
    }

    impl_gameobject!(CameraObject);
}
