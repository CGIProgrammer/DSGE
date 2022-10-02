use std::sync::Arc;

use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::device::{Device, Queue};
use vulkano::render_pass::{RenderPass, SubpassDescription, AttachmentDescription, AttachmentReference, RenderPassCreateInfo};
use vulkano::image::{ImageLayout, SampleCount};
use vulkano::command_buffer::CommandBufferUsage;

use crate::components::ProjectionUniformData;
use crate::components::GOTransformUniform;
use crate::game_object::AbstractVisual;
use crate::mesh::RcBox;
use crate::texture::TexturePixelFormat;
use crate::framebuffer::{FramebufferBinder, Framebuffer};

use super::geometry_pass::{DrawList, build_geometry_pass};

pub struct ShadowMapPass
{
    render_pass: Arc<RenderPass>,
    queue: Arc<Queue>,
    device: Arc<Device>,
}

impl ShadowMapPass
{
    pub fn new(queue : Arc<Queue>) -> Self
    {
        let desc = RenderPassCreateInfo {
            attachments: vec![
                AttachmentDescription {
                    format: Some(TexturePixelFormat::D16_UNORM),
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
        &self,
        shadow_map: &mut Framebuffer,
        light_projection_data: ProjectionUniformData,
        draw_list: &DrawList
    ) -> PrimaryAutoCommandBuffer
    {
        return build_geometry_pass(
            shadow_map,
            light_projection_data,
            draw_list,
            crate::material::MaterialShaderType::BaseShadowmap,
            self.render_pass.clone().first_subpass(),
            self.queue.clone()
        );
        /*let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        command_buffer_builder.bind_framebuffer(shadow_map, self.render_pass.clone(), false).unwrap();

        let mut last_mesh = -1;
        let mut last_material = -1;
        let subpass = self.render_pass.clone().first_subpass();
        let (mut shader_program, mut uniform_buffer) = draw_list[0].1.material().unwrap().lock().unwrap().base_shadowmap_shader(subpass.clone());
        let mut repeated_objects = Vec::with_capacity(draw_list.len());
        for (i, (transform, visual_component)) in draw_list.iter().enumerate()
        {
            let (mesh_id, material_id) = (visual_component.mesh_id(), visual_component.material_id());
            let new_mesh = last_mesh!=mesh_id;
            let new_material = last_material!=material_id;
            let new_unique_object = i==0 || new_mesh || new_material;
            let last_in_group = i==draw_list.len()-1 || draw_list[i+1].1.mesh_id()!=mesh_id || draw_list[i+1].1.material_id()!=material_id;
            if new_unique_object {
                (shader_program, uniform_buffer) = visual_component.material().unwrap().lock().unwrap().base_shadowmap_shader(subpass.clone());
                uniform_buffer.uniform(light_projection_data.clone(), crate::material::SHADER_CAMERA_SET, 0);
                repeated_objects.clear();
            }
            repeated_objects.push(transform.clone());
            if last_in_group {
                visual_component.on_geometry_pass(
                    &repeated_objects,
                    &light_projection_data,
                    &shader_program,
                    &mut uniform_buffer,
                    &mut command_buffer_builder,
                    new_mesh,
                    new_material,
                ).unwrap();
            }
            (last_mesh, last_material) = (mesh_id, material_id);
        }
        
        command_buffer_builder
            .end_render_pass().unwrap();
        
        command_buffer_builder.build().unwrap()*/
    }
}