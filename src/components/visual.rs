use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, SecondaryAutoCommandBuffer};
use vulkano::render_pass::Subpass;
use crate::vulkano::buffer::TypedBufferAccess;
use super::GOTransformUniform;
use crate::mesh::{MeshRef};
use crate::material::{MaterialRef};
use crate::references::*;

use crate::shader::{ShaderProgramBinder, ShaderProgram, ShaderProgramUniformBuffer};
use crate::mesh::{MeshCommandSet, VertexBufferRef, IndexBufferRef};

pub use crate::components::{
    ProjectionUniformData,
};

#[derive(Clone)]
pub struct MeshVisual
{
    mesh : MeshRef,
    material : MaterialRef,
    cast_shadow : bool,
    _vertex_buffer: VertexBufferRef,
    _index_buffer : IndexBufferRef,
    _base_geometry : Option<(ShaderProgram, ShaderProgramUniformBuffer)>,
    _base_shadowmap : Option<(ShaderProgram, ShaderProgramUniformBuffer)>,
}

impl MeshVisual
{
    pub fn new(mesh: MeshRef, material: MaterialRef, cast_shadow: bool) -> Self
    {
        let vbo = mesh.lock().vertex_buffer().unwrap().clone();
        let ibo = mesh.lock().index_buffer().unwrap().clone();
        Self {
            mesh : mesh.clone(),
            material : material,
            cast_shadow : cast_shadow,
            _vertex_buffer : vbo,
            _index_buffer : ibo,
            _base_geometry : None,
            _base_shadowmap : None,
        }
    }

    pub fn mesh(&self) -> &MeshRef
    {
        &self.mesh
    }

    pub fn material(&self) -> &MaterialRef
    {
        &self.material
    }
}

pub trait AbstractVisual: Sync + Send + 'static
{
    /// Выполняется при рендеринге на стадии геометрии
    fn on_geometry_pass(
        &mut self,
        _transform: GOTransformUniform,
        _camera_data: ProjectionUniformData,
        _subpass: Subpass,
        _acbb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        _new_mesh: bool,
        _new_material: bool
    ) -> Result<(), String>;

    /// Выполняется при рендеринге на стадии геометрии
    fn on_geometry_pass_secondary(
        &mut self,
        _transform: GOTransformUniform,
        _camera_data: ProjectionUniformData,
        _subpass: Subpass,
        _acbb: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
        _new_mesh: bool,
        _new_material: bool
    ) -> Result<(), String>;

    /// Выполняется при рендеринге на стадии карт теней
    fn on_shadowmap_pass(
        &mut self,
        _transform: GOTransformUniform,
        _camera_data: ProjectionUniformData,
        _subpass: Subpass,
        _acbb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        _new_mesh: bool,
        _new_material: bool
    ) -> Result<(), String>;

    /// Должно возвращать уникальный номер материала (необходимо для оптимизации)
    fn material_id(&self) -> i32;

    /// Должно возвращать уникальный номер полисетки (необходимо для оптимизации)
    fn mesh_id(&self) -> i32;
}

crate::impl_behaviour!(
    MeshVisual { }
);

impl AbstractVisual for MeshVisual
{
    fn on_geometry_pass(
        &mut self,
        transform: GOTransformUniform,
        camera_data: ProjectionUniformData,
        subpass: Subpass,
        acbb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        new_mesh: bool,
        new_material: bool
    ) -> Result<(), String>
    {
        let mut material = self.material.lock_write();
        let (shader, mut uniform_buffer) = material.base_shader(subpass);
        let mut shader = shader.clone();
        //drop(material);
        let _mesh = self.mesh.lock();
        let mesh = _mesh.clone();
        drop(_mesh);
        if new_material {
            uniform_buffer.uniform(camera_data, crate::material::SHADER_CAMERA_SET, 0);
            acbb
                .bind_shader_program(&shader).unwrap()
                .bind_shader_uniforms(&mut uniform_buffer, false)?;
        } else {
            //acbb.bind_shader_uniforms(&mut uniform_buffer, true)?;
        };
        acbb.bind_uniform_constant(&mut shader, transform)?;
        if new_mesh {
            acbb.bind_mesh(&mesh)?;
        }
        else
        {
            acbb.draw_mesh(&mesh)?;
        };
        Ok(())
    }

    fn on_geometry_pass_secondary(
        &mut self,
        transform: GOTransformUniform,
        camera_data: ProjectionUniformData,
        subpass: Subpass,
        acbb: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
        new_mesh: bool,
        new_material: bool
    ) -> Result<(), String>
    {
        let (shader, uniform_buffer) = match self._base_geometry {
            Some((ref mut shader, ref mut uniform_buffer)) => (shader, uniform_buffer),
            None => {
                let mut mat = self.material.lock();
                let (sh, ub) = mat.base_shader(subpass);
                self._base_geometry = Some((sh.clone(), ub.clone()));
                match self._base_geometry {
                    Some((ref mut shader, ref mut uniform_buffer)) => (shader, uniform_buffer),
                    None => panic!("Этот фрагмент не должен выполняться")
                }
            }
        };
        if new_material {
            uniform_buffer.uniform(camera_data, crate::material::SHADER_CAMERA_SET, 0);
            acbb
                .bind_shader_program(&shader).unwrap()
                .bind_shader_uniforms(uniform_buffer, false)?;
        };
        acbb.bind_uniform_constant(shader, transform)?;
        if new_mesh {
            acbb
                .bind_vertex_buffers(0, self._vertex_buffer.clone())
                .bind_index_buffer(self._index_buffer.clone())
                .draw_indexed(self._index_buffer.len() as u32, 1, 0, 0, 0).unwrap();
        }
        else
        {
            acbb
                .draw_indexed(self._index_buffer.len() as u32, 1, 0, 0, 0).unwrap();
        };
        Ok(())
    }

    fn on_shadowmap_pass(
        &mut self,
        transform: GOTransformUniform,
        camera_data: ProjectionUniformData,
        subpass: Subpass,
        acbb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        new_mesh: bool,
        new_material: bool
    ) -> Result<(), String>
    {
        if !self.cast_shadow {
            return Ok(());
        }
        let mut material = self.material.lock_write();
        let (shader, mut uniform_buffer) = material.base_shadowmap_shader(subpass);
        let mut shader = shader.clone();
        //drop(material);
        let _mesh = self.mesh.lock();
        let mesh = _mesh.clone();
        drop(_mesh);
        if new_material {
            uniform_buffer.uniform(camera_data, crate::material::SHADER_CAMERA_SET, 0);
            acbb
                .bind_shader_program(&shader).unwrap()
                .bind_shader_uniforms(&mut uniform_buffer, false)?
        } else {
            acbb.bind_shader_uniforms(&mut uniform_buffer, true)?
        };
        acbb.bind_uniform_constant(&mut shader, transform)?;
        if new_mesh {
            acbb.bind_mesh(&mesh)?;
        }
        else
        {
            acbb.draw_mesh(&mesh)?;
        };
        Ok(())
    }

    fn mesh_id(&self) -> i32 {
        self.mesh.box_id()
    }

    fn material_id(&self) -> i32 {
        self.material.box_id()
    }
}