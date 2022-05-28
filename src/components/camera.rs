use crate::shader::ShaderStructUniform;
use crate::texture::TextureRef;
use bytemuck::{Pod, Zeroable};

use crate::{
    types::Mat4,
};
use super::{GameObject, Component};


#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
/// Структура для передачи данных шейдерной программе
pub struct CameraUniformData
{
    pub transform : [f32; 16],
    pub transform_prev : [f32; 16],
    pub transform_inverted : [f32; 16],
    pub transform_prev_inverted : [f32; 16],
    pub projection : [f32; 16],
    pub projection_inverted : [f32; 16],
}

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

    pub fn uniform_data(&self, obj: &GameObject) -> CameraUniformData
    {
        let transform = obj.transform();
        CameraUniformData {
            transform : transform.global_for_render.as_slice().try_into().unwrap(),
            transform_prev : transform.global_for_render_prev.as_slice().try_into().unwrap(),
            transform_inverted : transform.global_for_render.try_inverse().unwrap().as_slice().try_into().unwrap(),
            transform_prev_inverted : transform.global_for_render_prev.try_inverse().unwrap().as_slice().try_into().unwrap(),
            projection : self.projection.as_slice().try_into().unwrap(),
            projection_inverted : self.projection.try_inverse().unwrap().as_slice().try_into().unwrap(),
        }
    }
}


impl Default for CameraUniformData
{
    fn default() -> Self
    {
        let proj = nalgebra::Perspective3::new(1.0, 80.0 * 3.1415926535 / 180.0, 0.1, 100.0).as_matrix().clone();
        Self {
            transform : Mat4::identity().as_slice().try_into().unwrap(),
            transform_prev : Mat4::identity().as_slice().try_into().unwrap(),
            transform_inverted : Mat4::identity().as_slice().try_into().unwrap(),
            transform_prev_inverted : Mat4::identity().as_slice().try_into().unwrap(),
            projection : proj.as_slice().try_into().unwrap(),
            projection_inverted : proj.try_inverse().unwrap().as_slice().try_into().unwrap(),
        }
    }
}

impl ShaderStructUniform for CameraUniformData
{
    fn glsl_type_name() -> String
    {
        String::from("Camera")
    }

    fn structure() -> String
    {
        String::from("{
            mat4 transform;
            mat4 transform_prev;
            mat4 transform_inverted;
            mat4 transform_prev_inverted;
            mat4 projection;
            mat4 projection_inverted;
        }")
    }
    
    fn texture(&self) -> Option<&TextureRef>
    {
        None
    }
}

impl Component for CameraComponent
{
    fn on_render_init(&mut self, _owner: &GameObject, _renderer: &mut crate::renderer::Renderer) -> Result<(), ()>
    {
        _renderer.update_camera_data(self.uniform_data(_owner));
        Ok(())
    }
}