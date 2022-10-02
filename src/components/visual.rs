use std::sync::Arc;

use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, SecondaryAutoCommandBuffer};
use vulkano::render_pass::Subpass;
use crate::references::MutexLockBox;
use crate::utils::RefId;
use super::GOTransformUniform;
use crate::mesh::{MeshRef, MeshView};
use crate::material::{MaterialRef};

use crate::shader::{ShaderProgram, ShaderProgramUniformBuffer};

pub use crate::components::{
    ProjectionUniformData,
};

#[derive(Clone)]
pub struct MeshVisual
{
    mesh : MeshRef,
    material : MaterialRef,
    cast_shadow : bool,
}

impl MeshVisual
{
    pub fn new(mesh: MeshRef, material: MaterialRef, cast_shadow: bool) -> Self
    {
        Self {
            mesh : mesh.clone(),
            material : material,
            cast_shadow : cast_shadow,
        }
    }

    #[inline(always)]
    pub fn mesh(&self) -> &MeshRef
    {
        &self.mesh
    }

    #[inline(always)]
    pub fn material(&self) -> &MaterialRef
    {
        &self.material
    }

    #[inline(always)]
    pub fn material_id(&self) -> i32
    {
        self.material.box_id()
    }
}

pub trait AbstractVisual: Sync + Send + 'static
{
    /// Выполняется при рендеринге на стадии геометрии
    /*fn on_geometry_pass(
        &mut self,
        _transform: &GOTransformUniform,
        _camera_data: &ProjectionUniformData,
        _subpass: &Subpass,
        _acbb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        _new_mesh: bool,
        _new_material: bool
    ) -> Result<(), String>;*/

    /// Выполняется при рендеринге на стадии геометрии
    fn on_geometry_pass_secondary(
        &self,
        _transform: &GOTransformUniform,
        _camera_data: &ProjectionUniformData,
        _subpass: &Subpass,
        _acbb: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
        _new_mesh: bool,
        _new_material: bool
    ) -> Result<(), String>;

    /// Выполняется при рендеринге на стадии карт теней
    fn on_geometry_pass(
        &self,
        camera_data: &ProjectionUniformData,
        instance_buffer: Arc<CpuAccessibleBuffer<[GOTransformUniform]>>,
        obj_index: u32,
        instance_count: u32,
        shader: &ShaderProgram,
        uniform_buffer: &mut ShaderProgramUniformBuffer,
        acbb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        new_mesh: bool,
        new_material: bool
    ) -> Result<(), String>;

    /// Должно возвращать уникальный номер материала (необходимо для оптимизации)
    fn material_id(&self) -> i32;

    /// Должно возвращать уникальный номер полисетки (необходимо для оптимизации)
    fn mesh_id(&self) -> i32;

    fn mesh(&self) -> Option<MeshRef>;

    fn material(&self) -> Option<MaterialRef>;
}

crate::impl_behaviour!(
    MeshVisual { }
);

impl RefId for MeshVisual {}
