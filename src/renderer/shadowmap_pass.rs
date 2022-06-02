use std::sync::Arc;

use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::device::{Device, Queue};
use vulkano::render_pass::{RenderPass, SubpassDescription, AttachmentDescription, AttachmentReference, RenderPassCreateInfo};
use vulkano::image::{ImageLayout, SampleCount};
use vulkano::command_buffer::CommandBufferUsage;

use crate::components::ProjectionUniformData;
use crate::components::{GOTransformUniform, Component};
use crate::mesh::RcBox;
use crate::texture::TexturePixelFormat;
use crate::framebuffer::{FramebufferBinder, Framebuffer};

pub struct ShadowMapPass
{
    render_pass: Arc<RenderPass>,
    queue: Arc<Queue>,
    device: Arc<Device>,
}

impl ShadowMapPass
{
    fn new(queue : Arc<Queue>) -> Self
    {
        let desc = RenderPassCreateInfo {
            attachments: vec![
                AttachmentDescription {
                    format: Some(TexturePixelFormat::D16_UNORM_S8_UINT),
                    samples: SampleCount::Sample1,
                    load_op: vulkano::render_pass::LoadOp::Clear,
                    store_op: vulkano::render_pass::StoreOp::Store,
                    stencil_load_op: vulkano::render_pass::LoadOp::Clear,
                    stencil_store_op: vulkano::render_pass::StoreOp::Store,
                    initial_layout: ImageLayout::DepthStencilAttachmentOptimal,
                    final_layout: ImageLayout::DepthStencilAttachmentOptimal,
                    ..Default::default()
                }
            ],
            subpasses: vec![SubpassDescription {
                color_attachments: vec![],
                depth_stencil_attachment: Some(AttachmentReference{
                    attachment: 0,
                    layout: ImageLayout::DepthStencilAttachmentOptimal,
                    ..Default::default()
                }),
                ..Default::default()
            }],
            ..Default::default()
        };

        Self {
            render_pass: RenderPass::new(queue.device().clone(), desc).unwrap(),
            queue: queue.clone(),
            device: queue.device().clone()
        }
    }

    pub fn build_shadow_map_pass(
        &mut self,
        shadow_map: &mut Framebuffer,
        light_projection_data: ProjectionUniformData,
        draw_list: Vec<(GOTransformUniform, RcBox<dyn Component>)>
    ) -> PrimaryAutoCommandBuffer
    {
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        command_buffer_builder.bind_framebuffer(shadow_map, self.render_pass.clone(), false).unwrap();

        let mut last_mesh = -1;
        let mut last_material = -1;
        let subpass = self.render_pass.clone().first_subpass();
        for (transform, visual_component) in draw_list
        {
            let mut component = visual_component.lock().unwrap();
            let (mesh_id, material_id) = (component.mesh_id(), component.material_id());
            component.on_shadowmap_pass(
                transform,
                light_projection_data,
                subpass.clone(),
                &mut command_buffer_builder,
                mesh_id != last_mesh,
                material_id != last_material,
            ).unwrap();
            (last_mesh, last_material) = (mesh_id, material_id);
        }
        
        command_buffer_builder
            .end_render_pass().unwrap();
        
        command_buffer_builder.build().unwrap()
    }
}