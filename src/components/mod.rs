/// Компоненты для `GameObject`
/// Пока в зачаточном состоянии
use std::any::Any;
use bytemuck::{Pod, Zeroable};

use vulkano::render_pass::Subpass;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};

pub mod camera;
pub mod visual;
pub mod light;

use crate::renderer::{PostprocessingPass, Renderer};
use crate::types::Mat4;
use crate::shader::ShaderStructUniform;
use crate::texture::Texture;

pub use crate::game_object::{GameObject, GOTransformUniform};
pub use camera::CameraComponent;
pub use visual::MeshVisual;
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

/// Component описывает поведение объекта
pub trait Component: 'static
{
    /// Выполняется при зарождении объекта на сцене
    fn on_start(&mut self, _owner: &mut GameObject)
    {
        
    }

    /// Выполняется на каждой итерации игры
    fn on_loop(&mut self, _owner: &mut GameObject)
    {
        
    }

    /// Выполняется при рендеринге на стадии геометрии
    fn on_geometry_pass(
        &mut self,
        _transform: GOTransformUniform,
        _camera_data: ProjectionUniformData,
        _subpass: Subpass,
        _acbb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        _new_mesh: bool,
        _new_material: bool
    ) -> Result<(), String>
    { 
        Err(format!("Объект {} не поддерживает отображение", std::any::type_name::<Self>()))
    }

    /// Выполняется при рендеринге на стадии карт теней
    fn on_shadowmap_pass(
        &mut self,
        _transform: GOTransformUniform,
        _camera_data: ProjectionUniformData,
        _subpass: Subpass,
        _acbb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        _new_mesh: bool,
        _new_material: bool
    ) -> Result<(), String>
    { 
        Err(format!("Объект {} не поддерживает отбрасывание теней", std::any::type_name::<Self>()))
    }

    /// Должно возвращать уникальный номер материала (необходимо для оптимизации)
    fn material_id(&self) -> i32
    {
        -1
    }

    /// Должно возвращать уникальный номер полисетки (необходимо для оптимизации)
    fn mesh_id(&self) -> i32
    {
        -1
    }

    /// Должно возвращать ссылку на самого себя в динамическом типе
    fn as_any(&self) -> &dyn Any;

    fn on_postprocess(&mut self, _owner: &GameObject, _postprocessor: &mut PostprocessingPass) -> Result<(), ()>
    {
        Err(())
    }
}

//pub trait DynamicComponent: Any + Component + 'static {}

