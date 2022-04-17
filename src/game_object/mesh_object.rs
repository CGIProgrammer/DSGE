use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
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
}

impl AbstractVisualObject for MeshObject {}

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