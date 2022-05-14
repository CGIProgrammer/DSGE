use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, SecondaryAutoCommandBuffer};
use vulkano::render_pass::RenderPass;

use super::{GOTransform, GameObject};
use super::impl_gameobject;
use crate::mesh::{MeshRef};
use crate::material::{MaterialRef};
use crate::references::*;

use crate::shader::ShaderProgramBinder;
use crate::mesh::MeshBinder;

pub use crate::components::{
    visual::AbstractVisual,
    AbstractVisualObject,
    camera::CameraUniformData
};

pub struct MeshObject
{
    transform : GOTransform,
    mesh : MeshRef,
    material : MaterialRef
}

impl MeshObject
{
    pub fn new(mesh: MeshRef, material: MaterialRef) -> RcBox<dyn GameObject>
    {
        let result = Self {
            transform : GOTransform::identity(),
            mesh : mesh,
            material : material
        };
        let result = RcBox::construct(result);
        result.take_mut().transform.set_owner(result.clone());
        result
    }
	
	fn fork_inner(&self) -> RcBox<dyn GameObject>
	{
		let result = Self {
            transform : self.transform.clone(),
            mesh : self.mesh.clone(),
            material : self.material.clone()
        };
        let result = RcBox::construct(result);
        result.take_mut().transform.set_owner(result.clone());
        result
	}
}

impl AbstractVisualObject for MeshObject {}

impl AbstractVisual for MeshObject
{
    fn draw_secondary(
        &self,
        mut _acbb : &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
        camera: CameraUniformData,
        render_pass: Arc<RenderPass>,
        subpass_id: u32,
        last_mesh_and_material: (i32, i32)
    ) -> Result<(i32, i32), String>
    {
        //let timer = std::time::SystemTime::now();
        let mesh_id = self.mesh.box_id();
        let material_id = self.material.box_id();
        let prev_mesh_id = last_mesh_and_material.0;
        let prevmaterial_id = last_mesh_and_material.1;
        let mut material = self.material.take_mut();
        let mesh = self.mesh.take();
        let shader = material.base_shader(render_pass, subpass_id);
        //shader.uniform(self.transform().uniform_value(), crate::material::SHADER_TRANSFORM_SET, 0);
        _acbb =
        if material_id != prevmaterial_id {
            //shader.clear_uniform_set(crate::material::SHADER_CAMERA_SET);
            shader.uniform(camera, crate::material::SHADER_CAMERA_SET, 0);
            //shader.build_uniform_sets(&[crate::material::SHADER_CAMERA_SET]);
            _acbb
                .bind_shader_program(shader)?
                .bind_shader_uniforms(shader, false)?
        } else {
            _acbb.bind_shader_uniforms(shader, true)?
        };
        _acbb.bind_uniform_constant(shader, self.transform().uniform_value())?;
        if mesh_id != prev_mesh_id {
            _acbb.bind_mesh(&*mesh)?;
        }
        else
        {
            _acbb.draw_mesh(&*mesh)?;
        }
        drop(shader);
        drop(material);
        drop(mesh);

        Ok((mesh_id, material_id))
    }
    
    fn draw(
        &self,
        mut _acbb : &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        camera: CameraUniformData,
        render_pass: Arc<RenderPass>,
        subpass_id: u32,
        last_mesh_and_material: (i32, i32)
    ) -> Result<(i32, i32), String>
    {
        //let timer = std::time::SystemTime::now();
        let mesh_id = self.mesh.box_id();
        let material_id = self.material.box_id();
        let prev_mesh_id = last_mesh_and_material.0;
        let prevmaterial_id = last_mesh_and_material.1;
        let mut material = self.material.take_mut();
        let mesh = self.mesh.take();
        let shader = material.base_shader(render_pass, subpass_id);
        //shader.uniform(self.transform().uniform_value(), crate::material::SHADER_TRANSFORM_SET, 0);
        _acbb =
        if material_id != prevmaterial_id
        {
            //shader.clear_uniform_set(crate::material::SHADER_CAMERA_SET);
            shader.uniform(camera, crate::material::SHADER_CAMERA_SET, 0);
            //shader.build_uniform_sets(&[crate::material::SHADER_CAMERA_SET]);
            _acbb
                .bind_shader_program(shader)?
                .bind_shader_uniforms(shader, false)?
        }
        else
        {
            _acbb.bind_shader_uniforms(shader, true)?
        };
        _acbb.bind_uniform_constant(shader, self.transform().uniform_value())?;
        //_acbb.push_constants();
        if mesh_id != prev_mesh_id {
            _acbb.bind_mesh(&*mesh)?;
        }
        else {
            _acbb.draw_mesh(&*mesh)?;
        }
        drop(shader);
        drop(material);
        drop(mesh);

        Ok((mesh_id, material_id))
    }
}

impl GameObject for MeshObject
{
    fn visual(&self) -> Option<&dyn super::AbstractVisualObject>
    {
        Some(self)
    }

    fn visual_mut(&mut self) -> Option<&mut dyn super::AbstractVisualObject>
    {
        Some(self)
    }

    fn camera(&self) -> Option<&dyn super::AbstractCameraObject>
    {
        None
    }

    fn camera_mut(&mut self) -> Option<&mut dyn super::AbstractCameraObject>
    {
        None
    }

    impl_gameobject!(MeshObject);
}