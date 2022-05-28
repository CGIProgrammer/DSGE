use crate::components::Component;

use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};

use super::{GameObject, Renderer};
use crate::mesh::{MeshRef};
use crate::material::{MaterialRef};
use crate::references::*;

use crate::shader::ShaderProgramBinder;
use crate::mesh::MeshBinder;

pub use crate::components::{
    camera::CameraUniformData
};

#[derive(Clone)]
pub struct MeshVisual
{
    mesh : MeshRef,
    material : MaterialRef
}

impl MeshVisual
{
    pub fn new(mesh: MeshRef, material: MaterialRef) -> Self
    {
        Self {
            mesh : mesh,
            material : material
        }
    }
}

impl Component for MeshVisual
{
    fn on_geometry_pass(
        &mut self,
        owner: &GameObject,
        renderer: &mut Renderer,
        acbb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        last_mesh_and_material: (i32, i32)
    ) -> Result<(i32, i32), String>
    {
        let subpass = renderer.geometry_subpass();
        let camera = renderer.camera_data();
        let mesh_id = self.mesh.box_id();
        let material_id = self.material.box_id();
        let prev_mesh_id = last_mesh_and_material.0;
        let prevmaterial_id = last_mesh_and_material.1;
        let mut material = self.material.take_mut();
        let (shader, mut uniform_buffer) = material.base_shader(subpass);
        let mut shader = shader.clone();
        drop(material);
        let _mesh = self.mesh.take();
        let mesh = _mesh.clone();
        drop(_mesh);
        if material_id != prevmaterial_id {
            uniform_buffer.uniform(camera, crate::material::SHADER_CAMERA_SET, 0);
            acbb
                .bind_shader_program(&shader)?
                .bind_shader_uniforms(&mut uniform_buffer, false)?
        } else {
            acbb.bind_shader_uniforms(&mut uniform_buffer, true)?
        };
        acbb.bind_uniform_constant(&mut shader, owner.transform().uniform_value())?;
        if mesh_id != prev_mesh_id {
            acbb.bind_mesh(&mesh)?;
        }
        else
        {
            acbb.draw_mesh(&mesh)?;
        }
        Ok((mesh_id, material_id))
    }
}