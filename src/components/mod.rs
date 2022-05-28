/// Компоненты для `GameObject`
/// Пока в зачаточном состоянии

pub mod camera;
pub mod visual;
pub use crate::game_object::GameObject;
use crate::renderer::{postprocessor::Postprocessor, Renderer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};

pub trait Component: std::any::Any + Clone
{
    fn on_start(&mut self, _owner: &GameObject) -> Result<(), ()>
    {
        Err(())
    }

    fn on_loop(&mut self, _owner: &GameObject) -> Result<(), ()>
    {
        Err(())
    }

    fn on_render_init(&mut self, _owner: &GameObject, _renderer: &mut Renderer) -> Result<(), ()>
    {
        Err(())
    }

    fn on_geometry_pass(
        &mut self,
        _owner: &GameObject,
        _renderer: &mut Renderer,
        _acbb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        _last_mesh_and_material: (i32, i32)) -> Result<(i32, i32), String>
    {
        Err(format!("Объект {} не может быть использован на геометрической стадии рендеринга", std::any::type_name::<Self>()))
    }

    fn on_postprocess(&mut self, _owner: &GameObject, _postprocessor: &mut Postprocessor) -> Result<(), ()>
    {
        Err(())
    }
}

