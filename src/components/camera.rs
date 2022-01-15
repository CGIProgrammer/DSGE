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