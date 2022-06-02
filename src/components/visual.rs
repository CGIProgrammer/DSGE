use std::any::Any;

use crate::components::Component;
use crate::game_object::GOTransform;

use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::render_pass::Subpass;

use super::{GameObject, Renderer, GOTransformUniform};
use crate::mesh::{MeshRef};
use crate::material::{MaterialRef};
use crate::references::*;

use crate::shader::ShaderProgramBinder;
use crate::mesh::MeshBinder;

pub use crate::components::{
    ProjectionUniformData,
};

#[derive(Clone)]
pub struct MeshVisual
{
    mesh : MeshRef,
    material : MaterialRef,
    cast_shadow : bool
}

impl MeshVisual
{
    pub fn new(mesh: MeshRef, material: MaterialRef, cast_shadow: bool) -> Self
    {
        Self {
            mesh : mesh,
            material : material,
            cast_shadow : cast_shadow
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

impl Component for MeshVisual
{
    fn as_any(&self) -> &dyn Any
    {
        self
    }

    /*fn on_render_init(&mut self, owner: &GameObject, renderer: &mut Renderer) -> Result<ProjectionUniformData, ()>
    {
        //renderer.add_renderable_component(owner.transform().uniform_value(), self);
        Ok(())
    }*/

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
        let mut material = self.material.take_mut();
        let (shader, mut uniform_buffer) = material.base_shader(subpass);
        let mut shader = shader.clone();
        drop(material);
        let _mesh = self.mesh.take();
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
        let mut material = self.material.take_mut();
        let (shader, mut uniform_buffer) = material.base_shadowmap_shader(subpass);
        let mut shader = shader.clone();
        drop(material);
        let _mesh = self.mesh.take();
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