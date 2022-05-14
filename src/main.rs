extern crate winit;
extern crate vulkano;
extern crate bytemuck;
extern crate half;

mod teapot;
mod time;

use std::sync::Arc;
use std::time::SystemTime;

use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::Version;
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent, DeviceEvent};
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
        
        // Создание камеры
        let camera = CameraObject::new(1.0, 85.0 * 3.1415926535 / 180.0, 0.1, 100.0);
        renderer.set_camera(camera);
        renderer.update_swapchain();

        // Загрузка теустуры
        let mut texture = Texture::from_file(renderer.queue().clone(), "data/texture/image_img.dds").unwrap();
        texture.set_anisotropy(Some(16.0));
        texture.set_mipmap(MipmapMode::Linear);
        texture.set_mag_filter(TextureFilter::Linear);
        texture.update_sampler();
        //texture.save(renderer.queue().clone(), "./dds_to.png");

        // Загрузка полисеток
        let mut monkey = Mesh::builder("monkey");
        //monkey.push_from_file("data/mesh/cube.mesh");
        monkey.push_from_file("data/mesh/monkey.mesh");
        let monkey = monkey.build_mutex(renderer.queue().clone()).unwrap();
        
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

        // Создание объектов
        let ob = MeshObject::new(monkey.clone(), material.clone());
        let mut objects = Vec::new();
        for row in -7..=7 {
            for col in -12..=12 {
                let ob = ob.lock().unwrap().fork();
                let mut _ob = ob.lock().unwrap();
                let transform = _ob.transform_mut();
                transform.local = nalgebra::Matrix4::from_euler_angles(0.0, 0.0, std::f32::consts::PI);
                for i in 0..15 {
                    transform.local[i] *= 0.05;
                }
                transform.local[12] = (col as f32) / 5.0;
                transform.local[13] = (row as f32) / 5.0;
                transform.local[14] = -1.0;
                drop(transform);
                drop(_ob);
                objects.push(ob);
            }
        }

        Ok(Self {
            renderer: renderer,
            event_pump: event_loop,
            root_objects: objects,
            counter: 0.0
        })
    }

    pub fn event_loop(mut self) -> Arc<EventLoop<()>>
    {
        let mut timer = time::Timer::new();
		let mut fps_timer = SystemTime::now();
		let mut frames = 0u32;
        let mut rpt = 0.0f64;
        let mut ppt = 0.0f64;
        let mut ft = 0.0f64;
        let mut take_screenshot = false;
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
                }
                Event::DeviceEvent {
                    event: DeviceEvent::Key {
                        0: input
                    }, ..
                } => {
                    match input.virtual_keycode.as_ref().unwrap() {
                        winit::event::VirtualKeyCode::F12 => {
                            match input.state {
                                winit::event::ElementState::Pressed => {
                                    println!("Скриншот");
                                    take_screenshot = true;
                                },
                                _ => {}
                            };
                        },
                        _ => {}
                    };
                }
                ,
                Event::RedrawEventsCleared => {
                    //println!("{}", frames);
                    let tu = timer.next_frame();
                    self.counter = tu.uptime;
                    let frame_timer = SystemTime::now();
                    self.renderer.update_timer(&tu);
                    let renderpass_timer = SystemTime::now();
                    self.renderer.begin_geametry_pass();
                    for obj in &self.root_objects{
                        obj.lock().unwrap().next_frame();
                        self.renderer.draw(obj.clone());
                    }
                    let renderpass_time = renderpass_timer.elapsed().unwrap().as_secs_f64();
                    let postprocess_timer = SystemTime::now();
                    self.renderer.wait();
                    if take_screenshot {
                        take_screenshot = false;
                        let img = self.renderer.postprocessor().get_output("swapchain_out".to_string()).unwrap();
                        img.take().save(self.renderer.queue().clone(), "./screenshot.png");
                    }
                    self.renderer.end_frame();
                    let postprocess_time = postprocess_timer.elapsed().unwrap().as_secs_f64();
                    let frame_time = frame_timer.elapsed().unwrap().as_secs_f64();
					frames += 1;
                    ppt += postprocess_time;
                    rpt += renderpass_time;
                    ft  += frame_time;
					let t = fps_timer.elapsed().unwrap().as_secs_f64();
					if t >= 1.0 {
						println!("{} FPS. frame_time = {:.3}ms, rp_time = {:.3}ms ({:.3}%), pp_time = {:.3}ms ({:.3}%)",
                            frames,
                            ft  / (frames as f64) * 1000.0,
                            ppt / (frames as f64) * 1000.0, ppt / t * 100.0,
                            rpt / (frames as f64) * 1000.0, rpt / t * 100.0
                        );
						frames = 0;
                        ppt = 0.0;
                        rpt = 0.0;
                        ft  = 0.0;
						fps_timer = SystemTime::now();
					}
                    //panic!("А на сегодня всё.");
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