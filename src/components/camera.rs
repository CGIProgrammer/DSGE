use crate::types::Mat4;
use crate::shader::ShaderStructUniform;
use crate::texture::TextureRef;
use bytemuck::{Pod, Zeroable};

/// 
pub trait AbstractCamera
{
    fn projection(&self) -> Mat4;
    fn set_projection(&mut self, aspect: f32, fov: f32, znear: f32, zfar: f32);
    fn uniform_data(&self) -> CameraUniformData;
}

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