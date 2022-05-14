use super::camera::CameraUniformData;
use std::sync::Arc;

use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, SecondaryAutoCommandBuffer};
use vulkano::render_pass::RenderPass;

/// # Абстрактное визуальное представление
/// Придаёт внешний вид объекту `GameObject`.
pub trait AbstractVisual
{
    fn draw(
        &self,
        _acbb : &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        camera: CameraUniformData,
        render_pass: Arc<RenderPass>,
        subpass_id: u32,
        last_mesh_and_material: (i32, i32)
    ) -> Result<(i32, i32), String>;
    
    fn draw_secondary(
        &self,
        _acbb : &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
        camera: CameraUniformData,
        render_pass: Arc<RenderPass>,
        subpass_id: u32,
        last_mesh_and_material: (i32, i32)
    ) -> Result<(i32, i32), String>;
}
