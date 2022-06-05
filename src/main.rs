extern crate winit;
extern crate vulkano;
extern crate bytemuck;

mod teapot;
mod time;

use std::sync::Arc;
use std::time::SystemTime;

use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::Version;
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent, DeviceEvent, KeyboardInput};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::window::{WindowBuilder, Fullscreen, Window};
use winit::event::*;

use game_object::*;
use references::*;

mod game_logic;
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
mod scene_loader;

use scene_loader::read_scene;

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
    surface: Arc<Surface<Window>>,
    renderer: renderer::Renderer,
    event_pump: EventLoop<()>,
    root_objects: Vec<RcBox<GameObject>>,
    cusor_delta: [f32; 2],
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
        let mut renderer = renderer::Renderer::winit(vk_instance, surface.clone(), _vsync);
        //let mut renderer = renderer::Renderer::offscreen(vk_instance, [width, height]);
        let (objects, camera) = read_scene("data/scenes/Scene.scene", renderer.queue().clone());
        let monkey = objects.iter().find(|obj| obj.take().name() == "monkey").unwrap().clone();
        let motion = game_logic::motion_example::MotionExample::default();
        monkey.take().add_component(motion);
        for obj in &objects
        {
            let mut _obj = obj.take_mut();
            let components = _obj.get_all_components().clone();
            for comp in components {
                let mut _comp = comp.lock().unwrap();
                _comp.on_start(&mut *_obj);
            }
        }
        renderer.set_camera(camera.unwrap());
        renderer.update_swapchain();
        Ok(Self {
            surface: surface,
            renderer: renderer,
            event_pump: event_loop,
            root_objects: objects,
            cusor_delta: [0.0, 0.0],
            counter: 0.0
        })
    }

    pub fn event_loop(mut self) /*-> Arc<EventLoop<()>>*/
    {
        let mut timer = time::Timer::new();
		let mut fps_timer = SystemTime::now();
		let mut frames = 0u32;
        let mut rpt = 0.0f64;
        let mut ppt = 0.0f64;
        let mut ft = 0.0f64;
        let mut take_screenshot = false;
        //surface.window().set_cursor_grab(true);
        self.event_pump.run(move |event: Event<()>, _wtar: &EventLoopWindowTarget<()>, control_flow: &mut ControlFlow| {
            match event {
                Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {event: WindowEvent::Resized(_), ..} => {
                    self.renderer.update_swapchain()
                }
                Event::DeviceEvent { event, .. } => {
                    match event {
                        DeviceEvent::Key(input) => {
                            match input {
                                KeyboardInput { virtual_keycode: Some(VirtualKeyCode::F12), state: ElementState::Pressed, .. } => {
                                    take_screenshot = true;
                                },
                                _ => ()
                            }
                        },
                        DeviceEvent::Button { button, state } => {
                            println!("Кнопка {}", button);
                            match (button, state) {
                                (1, ElementState::Pressed) => {
                                    drop(self.surface.window().set_cursor_grab(true));
                                },
                                (1, ElementState::Released) => {
                                    drop(self.surface.window().set_cursor_grab(false).unwrap());
                                },
                                _ => ()
                            }
                        },
                        DeviceEvent::MouseMotion { delta: (x, y) } => {
                            self.cusor_delta = [x as _, y as _];
                            println!("Мышь {}; {}", x, y);
                        }
                        _ => ()
                    }
                }
                ,
                Event::RedrawEventsCleared => {
                    //println!("{}", frames);
                    let tu = timer.next_frame();
                    self.counter = tu.uptime;
                    let frame_timer = SystemTime::now();
                    self.renderer.update_timer(tu);
                    let renderpass_timer = SystemTime::now();
                    self.renderer.begin_geametry_pass();
                    for obj in &self.root_objects
                    {
                        obj.take().next_frame();
                        self.renderer.draw(obj.clone());
                    }
                    let renderpass_time = renderpass_timer.elapsed().unwrap().as_secs_f64();
                    let postprocess_timer = SystemTime::now();
                    if take_screenshot {
                        self.renderer.wait();
                        take_screenshot = false;
                        let img = self.renderer.postprocessor().get_output("swapchain_out".to_string()).unwrap().clone();
                        img.save(self.renderer.queue().clone(), "./screenshot.png");
                    }
                    self.renderer.execute(std::collections::HashMap::new());
                    let postprocess_time = postprocess_timer.elapsed().unwrap().as_secs_f64();
                    let frame_time = frame_timer.elapsed().unwrap().as_secs_f64();
					frames += 1;
                    ppt += postprocess_time;
                    rpt += renderpass_time;
                    ft  += frame_time;
                    //println!("frame");
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
                },
                _ => { }
            }
        });
    }
}

fn main() {
    let app = Application::new("DSGE VK", 1280, 720, false, true).unwrap();
    app.event_loop();
}