extern crate dsge_vk;
extern crate winit;
extern crate vulkano;
extern crate bytemuck;

use dsge_vk::*;

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

use std::sync::Arc;
use std::time::SystemTime;

use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::Version;
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;
use winit::dpi::PhysicalPosition;
use winit::event::{Event, WindowEvent, DeviceEvent, KeyboardInput};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::window::{WindowBuilder, Fullscreen, Window};
use winit::event::*;

use game_object::*;
use references::*;
use renderer::Renderer;
use scene_loader::read_scene;

#[derive(Clone)]
pub struct Mouse
{
    surface: Arc<Surface<Window>>,
    mouse_delta: [i32; 2],
    cursor_position: [i32; 2],
    grabbed: bool,
    visible: bool,
}

impl Mouse
{
    fn new_with_surface(surface: Arc<Surface<Window>>) -> Self
    {
        Self {
            surface: surface,
            mouse_delta: [0, 0],
            cursor_position: [0, 0],
            grabbed: false,
            visible: true,
        }
    }

    pub fn set_cursor_position(&self, x: i32, y: i32) -> Result<(), String>
    {
        let cur_pos = PhysicalPosition::new(x, y);
        match self.surface.window().set_cursor_position(cur_pos) {
            Ok(_) => Ok(()),
            Err(error) => Err(format!("Ошибка установки курсора: {:?}", error))
        }
    }

    pub fn set_cursor_visible(&mut self, visible: bool)
    {
        self.visible = visible;
        self.surface.window().set_cursor_visible(visible);
    }

    pub fn set_cursor_grab(&mut self, grab: bool) -> Result<(), String>
    {
        
        match self.surface.window().set_cursor_grab(grab) {
            Ok(_) => {
                self.grabbed = grab;
                Ok(())
            },
            Err(error) => Err(format!("Ошибка установки курсора: {:?}", error))
        }
    }

    #[inline]
    pub fn cursor_grab(&mut self) -> bool
    {
        self.grabbed
    }

    #[inline]
    pub fn mouse_delta(&self) -> [i32; 2]
    {
        self.mouse_delta
    }

    #[inline]
    pub fn cursor_position(&self) -> [i32; 2]
    {
        self.cursor_position
    }
}

pub struct Time
{
    frame_time: f64,
    up_time: f64,
}

impl Default for Time
{
    fn default() -> Self {
        Self {
            frame_time: 1.0 / 60.0,
            up_time: 0.0
        }
    }
}

impl Time
{
    #[inline]
    pub fn time_delta(&self) -> f64
    {
        self.frame_time
    }

    #[inline]
    pub fn up_time(&self) -> f64
    {
        self.up_time
    }
}

/// Базовый пример приложения
pub struct App {
    surface: Arc<Surface<Window>>,
    renderer: Renderer,
    event_pump: Option<EventLoop<()>>,
    root_objects: Vec<RcBox<GameObject>>,
    mouse: RcBox<Mouse>,
    time: Time,
}

impl App
{
    pub fn time(&self) -> &Time
    {
        &self.time
    }

    pub fn mouse(&self) -> RcBox<Mouse>
    {
        self.mouse.clone()
    }
}

impl App {
    fn new(title: &str, width: u16, height: u16, fullscreen: bool, _vsync: bool) -> Result<Self, String>
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
        //glfw::Window
        
        // Инициализация рендера
        let mut renderer = Renderer::winit(vk_instance, surface.clone(), _vsync, true);
        //let mut renderer = renderer::Renderer::offscreen(vk_instance, [width, height]);
        let (objects, camera) = read_scene("data/scenes/cubes_fps_test.scene", renderer.queue().clone());
        let monkey = objects.iter().find(|obj| obj.take().name() == "monkey");
        match monkey {
            Some(monkey) => {
                let motion = game_logic::motion_example::MotionExample::default();
                monkey.take().add_component(motion);
            },
            None => ()
        }
        for obj in &objects
        {
            let components = obj.take_mut().get_all_components().clone();
            for comp in components {
                let mut _comp = comp.lock().unwrap();
                _comp.on_start(obj.clone());
            }
        }
        /*let ml = game_logic::mouse_look::MouseLook::default();
        camera.clone().unwrap().take().add_component(ml);*/
        renderer.set_camera(camera.unwrap());
        renderer.update_swapchain();
        Ok(Self {
            surface: surface.clone(),
            renderer: renderer,
            event_pump: Some(event_loop),
            root_objects: objects,
            mouse: RcBox::construct(Mouse::new_with_surface(surface)),
            time: Time::default()
        })
    }

    fn objects_behaviour_iteration(slf: &Self, obj: GameObjectRef)
    {
        let mut components = obj.take().get_all_components().clone();
        let children = obj.take().children().clone();
        for component in components
        {
            let mut cmp = component.lock().unwrap();
            cmp.on_loop(obj.clone());
        }
        
        for child in children
        {
            Self::objects_behaviour_iteration(slf, child.clone());
        }
    }

    fn event_loop(mut self) /*-> Arc<EventLoop<()>>*/
    {
        let mut timer = time::Timer::new();
        let mut take_screenshot = false;
        let mut grab_coords = [0i32, 0i32];
        //surface.window().set_cursor_grab(true);
        let event_pump;
        (event_pump, self.event_pump) = (self.event_pump, None);
        let app = RcBox::construct(self);
        let app2 = app.clone();
        let object_events_func = move || {
            let root_objects = app2.take().root_objects.clone();
            let camera = app2.take().renderer.camera().clone().unwrap();
            for obj in root_objects
            {
                Self::objects_behaviour_iteration(&*app2.take(), obj);
            }
            Self::objects_behaviour_iteration(&*app2.take(), camera);

        };
        event_pump.unwrap().run(move |event: Event<()>, _wtar: &EventLoopWindowTarget<()>, control_flow: &mut ControlFlow| {
            //*control_flow = ControlFlow::Wait;
            let a = app.clone();
            match event {
                Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {event: WindowEvent::Resized(_), ..} => {
                    a.take().renderer.update_swapchain()
                }
                Event::WindowEvent {event: WindowEvent::CursorMoved{position, ..}, ..} => {
                    a.take().mouse.take().cursor_position = [position.x as _, position.y as _]
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
                            match (button, state) {
                                (1, ElementState::Pressed) => {
                                    let slf = a.take();
                                    let mut mouse = slf.mouse.take();
                                    grab_coords = mouse.cursor_position;
                                    match mouse.set_cursor_grab(true) {
                                        Ok(_)  => {
                                            drop(mouse.set_cursor_visible(false));
                                        },
                                        Err(_) => ()
                                    };
                                },
                                (1, ElementState::Released) => {
                                    let slf = a.take();
                                    let mut mouse = slf.mouse.take();
                                    if mouse.cursor_grab() {
                                        match mouse.set_cursor_grab(false) {
                                            Ok(_)  => {
                                                drop(mouse.set_cursor_position(grab_coords[0], grab_coords[1]));
                                                mouse.set_cursor_visible(true);
                                            },
                                            Err(_) => ()
                                        };
                                    }
                                },
                                _ => ()
                            }
                        },
                        DeviceEvent::MouseMotion { delta: (x, y) } => {
                            a.take().mouse.take().mouse_delta = [x as _, y as _];
                        }
                        _ => ()
                    }
                }
                ,
                Event::RedrawEventsCleared => {
                    //println!("{}", frames);
                    let tu = timer.next_frame();
                    let mut slf = a.take();
                    slf.time.up_time = tu.uptime as _;
                    slf.time.frame_time = tu.delta as _;
                    slf.renderer.update_timer(tu);
                    slf.renderer.begin_geametry_pass();
                    drop(slf);
                    object_events_func();
                    let objects = a.take().root_objects.clone();
                    for obj in objects
                    {
                        obj.take().next_frame();
                        a.take().renderer.draw(obj);
                    }
                    if take_screenshot {
                        let mut slf = a.take();
                        slf.renderer.wait();
                        take_screenshot = false;
                        let img = slf.renderer.postprocessor().get_output("swapchain_out".to_string()).unwrap().clone();
                        img.save(slf.renderer.queue().clone(), "./screenshot.png");
                    }
                    a.take().renderer.execute(std::collections::HashMap::new());
                    a.take().mouse.take().mouse_delta = [0, 0];
                },
                _ => { }
            }
        })
    }
}

fn main() {
    let app = App::new("DSGE VK", 640, 360, false, false).unwrap();
    app.event_loop();
}