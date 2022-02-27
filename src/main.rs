extern crate winit;
extern crate vulkano;

mod teapot;
mod time;

use std::sync::Arc;

use vulkano::instance::Instance;
use vulkano::Version;
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::window::{WindowBuilder, Fullscreen};

#[macro_use]
mod shader;
mod mesh;
mod types;
mod glenums;
mod utils;
mod references;
mod framebuffer;
mod texture;
mod material;
mod game_object;
mod renderer;
mod components;

use mesh::*;
use texture::*;
use game_object::*;
use types::Vec4;

trait Radian
{
    fn rad(&self) -> Self;
}

impl Radian for f32
{
    fn rad(&self) -> Self
    {
        self*3.1415926535/180.0
    }
}

/// Базовый пример приложения
pub struct Application {
    renderer: renderer::Renderer,
    event_pump: EventLoop<()>,
    object : RcBox<MeshObject>,
    counter: f32
}

impl Application {
    pub fn new(title: &str, width: u16, height: u16, fullscreen: bool, _vsync: bool) -> Result<Self, String>
    {
        let required_extensions = vulkano_win::required_extensions();
        let vk_instance = Instance::new(None, Version::V1_2, &required_extensions, None).unwrap();
        
        let event_loop = EventLoop::new();
        let wsize = winit::dpi::PhysicalSize { width, height };
        let surface = WindowBuilder::new()
            .with_title(title)
            .with_fullscreen(if fullscreen { Some(Fullscreen::Borderless(None)) } else { None } )
            .with_inner_size(wsize)
            .build_vk_surface(&event_loop, vk_instance.clone())
            .unwrap();

        let mut renderer = renderer::Renderer::from_winit(vk_instance, surface, _vsync);
        let mut texture = Texture::from_file(renderer.queue().clone(), "image_img.png").unwrap();
        texture.set_anisotropy(16.0);
        texture.set_mipmap(MipmapMode::Linear);
        texture.set_mag_filter(TextureFilter::Linear);
        texture.update_sampler();

        let mut mesh = Mesh::builder("default teapot");
        mesh.push_teapot();
        let mesh = mesh.build_mutex(renderer.device().clone()).unwrap();
        
        let mut material = material::MaterialBuilder::start(renderer.device().clone());
        material
            .define("diffuse_map", "")
            .add_texture("fDiffuseMap", RcBox::construct(texture))
            .add_numeric_parameter("diffuse", [1.0, 1.0, 1.0, 1.0].into())
            .add_numeric_parameter("roughness", 1.0.into())
            .add_numeric_parameter("glow", 1.0.into())
            .add_numeric_parameter("metallic", 0.0.into());
        let material = material.build_mutex(renderer.device().clone());

        renderer.set_camera(RcBox::construct(CameraObject::new(1.0, 85.0 * 3.1415926535 / 180.0, 0.1, 100.0)));

        Ok(Self {
            renderer: renderer,
            event_pump: event_loop,
            object: RcBox::construct(MeshObject::new(mesh, material)),
            counter: 0.0
        })
    }

    pub fn event_loop(mut self) -> Arc<EventLoop<()>>
    {
        let mut timer = time::Timer::new();
        self.event_pump.run(move |event: Event<()>, _wtar: &EventLoopWindowTarget<()>, control_flow: &mut ControlFlow| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    self.renderer.update_swapchain()
                },
                Event::RedrawEventsCleared => {
                    let tu = timer.next_frame();
                    self.counter = tu.uptime;
                    let transform = types::Mat4::new(
                        1.0, 0.0, 0.0,  0.0,
                        0.0, 1.0, 0.0,  0.0,
                        0.0, 0.0, 1.0, -2.0,
                        0.0, 0.0, 0.0,  1.0,
                    ) * types::Mat4::new(
                         self.counter.cos(), 0.0, self.counter.sin(), 0.0,
                         0.0, 1.0, 0.0, 0.0,
                        -self.counter.sin(), 0.0, self.counter.cos(), 0.0,
                         0.0, 0.0, 0.0, 1.0,
                    );
                    let mut obj = self.object.take_mut();
                    let mut obj_transform = obj.transform_mut();
                    
                    obj_transform.global_for_render_prev = obj_transform.global_for_render;
                    obj_transform.global_for_render = transform;
                    obj_transform.global = transform;
                    drop(obj_transform);
                    drop(obj);
                    self.renderer.update_timer(&tu);
                    self.renderer.begin_geametry_pass();
                    self.renderer.draw(self.object.clone());
                    self.renderer.end_frame();
                },
                _ => { }
            }
        });
    }
}

fn main() {
    let app = Application::new("DSGE VK", 800, 600, false, false).unwrap();
    app.event_loop();
}