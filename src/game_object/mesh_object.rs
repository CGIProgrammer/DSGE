use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::render_pass::RenderPass;

use super::{GOTransform, GameObject};
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
    material : MaterialRef,
    pub parent : Option<RcBox<dyn GameObject>>,
    pub children : Vec::<RcBox<dyn GameObject>>
}

impl MeshObject
{
    pub fn new(mesh: MeshRef, material: MaterialRef) -> Self
    {
        Self {
            transform : GOTransform::identity(),
            mesh : mesh,
            material : material,
            parent : None,
            children : Vec::new(),
        }
    }
}

impl AbstractVisualObject for MeshObject {}

impl GameObject for MeshObject
{
    fn transform(&self) -> &GOTransform
    {
        &self.transform
    }
    
    fn transform_mut(&mut self) -> &mut GOTransform
    {
        &mut self.transform
    }

    fn apply_transform(&mut self)
    {
        for child in &mut self.children {
            child.lock().unwrap().apply_transform();
        }
    }
}

impl AbstractVisual for MeshObject
{
    fn draw(
        &self,
        mut _acbb : &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        camera: &CameraUniformData,
        render_pass: Arc<RenderPass>,
        subpass_id: u32
    ) -> Result<(), String>
    {
        let mut material = self.material.take_mut();
        let mesh = self.mesh.take();
        let shader = material.base_shader(render_pass, subpass_id);
        shader.uniform(&self.transform().uniform_value(), crate::material::SHADER_VARIABLES_SET, 0);
        shader.uniform(camera, crate::material::SHADER_VARIABLES_SET, 1);
        _acbb = match _acbb.bind_shader_program(shader)
        {
            Ok(acbb) => acbb,
            Err(e) => return Err(e)
        };
        _acbb = match _acbb.bind_shader_uniforms(shader)
        {
            Ok(acbb) => acbb,
            Err(e) => return Err(e)
        };
        _acbb = match _acbb.bind_mesh(&*mesh)
        {
            Ok(acbb) => acbb,
            Err(e) => return Err(e)
        };
        Ok(())
    }
}