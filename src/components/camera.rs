use crate::types::Mat4;
use crate::shader::ShaderStructUniform;
use crate::texture::TextureRef;

/// 
pub trait AbstractCamera
{
    fn projection(&self) -> Mat4;
    fn set_projection(&mut self, aspect: f32, fov: f32, znear: f32, zfar: f32);
    fn uniform_data(&self) -> CameraUniformData;
}

#[derive(Copy, Clone)]
/// Структура для передачи данных шейдерной программе
pub struct CameraUniformData
{
    pub transform : Mat4,
    pub transform_prev : Mat4,
    pub transform_inverted : Mat4,
    pub transform_prev_inverted : Mat4,
    pub projection : Mat4,
    pub projection_inverted : Mat4,
}

impl Default for CameraUniformData
{
    fn default() -> Self
    {
        let proj = nalgebra::Perspective3::new(1.0, 80.0 * 3.1415926535 / 180.0, 0.1, 100.0).as_matrix().clone();
        Self {
            transform : Mat4::identity(),
            transform_prev : Mat4::identity(),
            transform_inverted : Mat4::identity(),
            transform_prev_inverted : Mat4::identity(),
            projection : proj,
            projection_inverted : proj.try_inverse().unwrap(),
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