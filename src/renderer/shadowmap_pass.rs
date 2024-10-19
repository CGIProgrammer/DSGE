use std::sync::Arc;

use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::Queue;
use vulkano::image::{ImageLayout, SampleCount};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::render_pass::{
    AttachmentDescription, AttachmentReference, RenderPass, RenderPassCreateInfo,
    SubpassDescription,
};

use super::geometry_pass::{build_geometry_pass, DrawList};
use super::BumpMemoryAllocator;
use crate::command_buffer::CommandBufferFather;
use crate::components::ProjectionUniformData;
use crate::framebuffer::Framebuffer;
use crate::material::MaterialShaderProgramType;
use crate::texture::TexturePixelFormat;
use crate::time::UniformTime;

pub struct ShadowMapPass {
    render_pass: Arc<RenderPass>,
}

impl ShadowMapPass {
    pub fn new(queue: Arc<Queue>) -> Self {
        let desc = RenderPassCreateInfo {
            attachments: vec![AttachmentDescription {
                format: TexturePixelFormat::D16_UNORM,
                samples: SampleCount::Sample1,
                load_op: vulkano::render_pass::AttachmentLoadOp::Load,
                store_op: vulkano::render_pass::AttachmentStoreOp::Store,
                stencil_load_op: Some(vulkano::render_pass::AttachmentLoadOp::DontCare),
                stencil_store_op: Some(vulkano::render_pass::AttachmentStoreOp::DontCare),
                initial_layout: ImageLayout::DepthStencilAttachmentOptimal,
                final_layout: ImageLayout::DepthStencilReadOnlyOptimal,
                ..Default::default()
            }],
            subpasses: vec![SubpassDescription {
                color_attachments: vec![],
                depth_stencil_attachment: Some(AttachmentReference {
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
        }
    }

    pub fn build_shadow_map_pass(
        &self,
        shadow_map: &mut Framebuffer,
        light_projection_data: ProjectionUniformData,
        timer: UniformTime,
        draw_list: DrawList,
        command_buffer_father: &CommandBufferFather,
        allocator: Arc<BumpMemoryAllocator>,
        ds_allocator: Arc<StandardDescriptorSetAllocator>
    ) -> Result<Arc<PrimaryAutoCommandBuffer>, String> {
        build_geometry_pass(
            shadow_map,
            light_projection_data,
            timer,
            draw_list,
            MaterialShaderProgramType::base_shadowmap(),
            self.render_pass.clone().first_subpass(),
            command_buffer_father,
            allocator,
            ds_allocator
        )
    }
}
