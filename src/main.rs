extern crate winit;
extern crate vulkano;
extern crate bytemuck;

mod teapot;
mod time;

use std::sync::Arc;

use vulkano::instance::{Instance, InstanceCreateInfo};
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
#[macro_use]
mod game_object;
mod renderer;
mod components;

use mesh::*;
use texture::*;
use game_object::*;

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
    root_objects: Vec<RcBox<dyn GameObject>>,
    counter: f32
}

impl Application {
    pub fn new(title: &str, width: u16, height: u16, fullscreen: bool, _vsync: bool) -> Result<Self, String>
    {
        let required_extensions = vulkano_win::required_extensions();
        let vk_instance = Instance::new(InstanceCreateInfo {
            enabled_extensions: required_extensions,
            max_api_version: Some(Version::major_minor(1, 2)),
            ..Default::default()
        }).unwrap();
        
        let event_loop = EventLoop::new();
        let wsize = winit::dpi::PhysicalSize { width, height };
        let surface = WindowBuilder::new()
            .with_title(title)
            .with_fullscreen(if fullscreen { Some(Fullscreen::Borderless(None)) } else { None } )
            .with_inner_size(wsize)
            .build_vk_surface(&event_loop, vk_instance.clone())
            .unwrap();

        // Инициализация рендера
        let mut renderer = renderer::Renderer::from_winit(vk_instance, surface, _vsync);

        // Загрузка теустуры
        let mut texture = Texture::from_file(renderer.queue().clone(), "data/texture/image_img.dds").unwrap();
        texture.set_anisotropy(Some(1.0));
        texture.set_mipmap(MipmapMode::Linear);
        texture.set_mag_filter(TextureFilter::Linear);
        texture.update_sampler();

        // Загрузка полисеток
        let mut monkey_subdiv_1 = Mesh::builder("default teapot");
        monkey_subdiv_1.push_from_file("data/mesh/monkey_subdiv_1.mesh");
        //mesh.push_teapot();
        let monkey_subdiv_1 = monkey_subdiv_1.build_mutex(renderer.device().clone()).unwrap();
        let mut monkey = Mesh::builder("monkey");
        monkey.push_from_file("data/mesh/monkey_subdiv_1.mesh");
        let monkey = monkey.build_mutex(renderer.device().clone()).unwrap();
        
        // Создание материала
        let mut material = material::MaterialBuilder::start(renderer.device().clone());
        material
            .define("diffuse_map", "")
            .add_texture("fDiffuseMap", RcBox::construct(texture))
            .add_numeric_parameter("diffuse", [1.0, 1.0, 1.0, 1.0].into())
            .add_numeric_parameter("roughness", 1.0.into())
            .add_numeric_parameter("glow", 1.0.into())
            .add_numeric_parameter("metallic", 0.0.into());
        let material = material.build_mutex(renderer.device().clone());

        // Создание камеры
        let camera = CameraObject::new(1.0, 85.0 * 3.1415926535 / 180.0, 0.1, 100.0);
        renderer.set_camera(camera);

        // Создание объектов
        let o1 = MeshObject::new(monkey_subdiv_1, material.clone());
        let o2 = MeshObject::new(monkey.clone(), material.clone());
        let o2_satellite = MeshObject::new(monkey, material);
        o2_satellite.lock().unwrap().set_parent(o2.clone());
        o2_satellite.lock().unwrap().transform_mut().local[12] += 1.0;

        //o2.take_mut().se

        Ok(Self {
            renderer: renderer,
            event_pump: event_loop,
            root_objects: vec![o1, o2],
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
                        0.0, 0.0, 1.0, -3.0,
                        0.0, 0.0, 0.0,  1.0,
                    ) * types::Mat4::new(
                         self.counter.cos(), 0.0, self.counter.sin(), 0.0,
                         0.0, 1.0, 0.0, 0.0,
                        -self.counter.sin(), 0.0, self.counter.cos(), 0.0,
                         0.0, 0.0, 0.0, 1.0,
                    );
                    let mut obj = self.root_objects[0].lock().unwrap();
                    let mut monkey = self.root_objects[1].lock().unwrap();
                    let mut obj_transform = obj.transform_mut();
                    let mut mon_transform = monkey.transform_mut();
                    
                    let tr1 = types::Mat4::new(
                        1.0, 0.0, 0.0,  1.0,
                        0.0, 1.0, 0.0,  0.0,
                        0.0, 0.0, 1.0,  0.0,
                        0.0, 0.0, 0.0,  1.0,
                    ) * transform * 
                    types::Mat4::new(
                        0.5, 0.0, 0.0,  0.0,
                        0.0, 0.5, 0.0,  0.0,
                        0.0, 0.0, 0.5,  0.0,
                        0.0, 0.0, 0.0,  1.0,
                    );
                    let tr2 = types::Mat4::new(
                        1.0, 0.0, 0.0, -1.0,
                        0.0, 1.0, 0.0,  0.0,
                        0.0, 0.0, 1.0,  0.0,
                        0.0, 0.0, 0.0,  1.0,
                    ) * transform * 
                    types::Mat4::new(
                        0.5, 0.0, 0.0,  0.0,
                        0.0, 0.5, 0.0,  0.0,
                        0.0, 0.0, 0.5,  0.0,
                        0.0, 0.0, 0.0,  1.0,
                    );
                    //obj_transform.global = tr1;
                    //mon_transform.global = tr2;
                    obj_transform.local = tr1;
                    mon_transform.local = tr2;
                    drop(mon_transform);
                    drop(obj_transform);
                    drop(monkey);
                    drop(obj);

                    self.renderer.update_timer(&tu);
                    self.renderer.begin_geametry_pass();
                    for obj in &self.root_objects{
                        obj.lock().unwrap().next_frame();
                        self.renderer.draw(obj.clone());
                    }
                    self.renderer.end_frame();
                },
                _ => { }
            }
        });
    }
}

fn main() {
    let app = Application::new("DSGE VK", 320, 300, false, false).unwrap();
    app.event_loop();
}