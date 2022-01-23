use crate::types::Mat4;
use crate::shader::ShaderStructUniform;
use crate::texture::TextureRef;

#[derive(Copy, Clone)]
pub struct CameraUniform
{
    pub transform : Mat4,
    pub transform_prev : Mat4,
    pub transform_inverted : Mat4,
    pub transform_prev_inverted : Mat4,
    pub projection : Mat4,
    pub projection_inverted : Mat4,
}

#[allow(dead_code)]
impl CameraUniform
{
    pub fn identity(aspect: f32, fov: f32, znear: f32, zfar: f32) -> Self
    {
        let proj = nalgebra::Perspective3::new(aspect, fov * 3.1415926535 / 180.0, znear, zfar).as_matrix().clone();
        Self {
            transform : Mat4::identity(),
            transform_prev : Mat4::identity(),
            transform_inverted : Mat4::identity(),
            transform_prev_inverted : Mat4::identity(),
            projection : proj,
            projection_inverted : proj.try_inverse().unwrap(),
        }
    }

    pub fn set(&mut self, transform: Mat4)
    {
        self.transform = transform;
        self.transform_inverted = transform.try_inverse().unwrap();
    }

    pub fn save_transform(&mut self)
    {
        self.transform_prev = self.transform;
        self.transform_prev_inverted = self.transform_inverted;
    }
}

impl Default for CameraUniform
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

impl ShaderStructUniform for CameraUniform
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