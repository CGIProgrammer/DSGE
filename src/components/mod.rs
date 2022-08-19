/// Компоненты для `GameObject`
/// Пока в зачаточном состоянии
use bytemuck::{Pod, Zeroable};

pub mod camera;
pub mod visual;
pub mod light;

use crate::types::Mat4;
use crate::shader::ShaderStructUniform;
use crate::texture::Texture;

pub use crate::game_object::{GameObject, GameObjectRef, GOTransformUniform};
pub use camera::CameraComponent;
pub use visual::{MeshVisual, AbstractVisual};
pub use light::Light;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
/// Структура для передачи данных шейдерной программе
pub struct ProjectionUniformData
{
    pub transform : [f32; 16],
    pub transform_prev : [f32; 16],
    pub transform_inverted : [f32; 16],
    pub transform_prev_inverted : [f32; 16],
    pub projection : [f32; 16],
    pub projection_inverted : [f32; 16],
}

impl ProjectionUniformData
{
    pub fn full_matrix(&self) -> Mat4
    {
        let transform = Mat4::new(
            self.transform[0],  self.transform[1],  self.transform[2],  self.transform[3],
            self.transform[4],  self.transform[5],  self.transform[6],  self.transform[7],
            self.transform[8],  self.transform[9],  self.transform[10], self.transform[11],
            self.transform[12], self.transform[13], self.transform[14], self.transform[15],
        ).transpose();
        let projection = Mat4::new(
            self.projection[0],  self.projection[1],  self.projection[2],  self.projection[3],
            self.projection[4],  self.projection[5],  self.projection[6],  self.projection[7],
            self.projection[8],  self.projection[9],  self.projection[10], self.projection[11],
            self.projection[12], self.projection[13], self.projection[14], self.projection[15],
        ).transpose();
        projection * transform.try_inverse().unwrap()
    }

    pub fn full_matrix_inverted(&self) -> Mat4
    {
        self.full_matrix().try_inverse().unwrap()
    }
}

impl Default for ProjectionUniformData
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

impl ShaderStructUniform for ProjectionUniformData
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
    
    fn texture(&self) -> Option<&Texture>
    {
        None
    }
}