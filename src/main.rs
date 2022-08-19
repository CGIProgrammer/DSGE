extern crate dsge_vk;
extern crate winit;
extern crate vulkano;
extern crate bytemuck;

use dsge_vk::*;
use dsge_vk::game_logic::AbstractEvent;
use dsge_vk::game_logic::events::{*};
use dsge_vk::scene::{SceneRef,Scene};

use dsge_vk::types::FastProjection;
use game_logic::motion_example::*;
use game_logic::mouse_look::*;
use nalgebra::Perspective3;

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

use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::Version;
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;
use winit::dpi::PhysicalPosition;
use winit::event::{Event, WindowEvent, DeviceEvent, KeyboardInput};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::window::{WindowBuilder, Fullscreen, Window};
use winit::event::*;

use references::*;
use renderer::Renderer;

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
    renderer: Renderer,
    scene: SceneRef,
    event_pump: Option<EventLoop<()>>,
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
        
        // Инициализация рендера
        let mut renderer = Renderer::winit(vk_instance, surface.clone(), _vsync, false);
        //let mut renderer = renderer::Renderer::offscreen(vk_instance, [width, height]);
        let (scene, camera) = Scene::from_file("data/scenes/Scene.scene", renderer.queue().clone());
        let objects = scene.lock().root_objects();
        let monkey = objects.iter().find(|obj| obj.lock().name()=="monkey");
        let light = objects.iter().find(|obj| obj.lock().name()=="light");
        
        if let Some(monkey) = monkey {
            println!("name: {}", monkey.lock().name());
            let motion = Spinning::default();
            monkey.lock().add_component(motion);

        };
        //let camera = objects.iter().find(|obj| obj.lock().name()=="Camera");
        if let Some(camera) = camera.clone() {
            println!("name: {}", camera.lock().name());
            let motion = MouseLook::new(0.0025);
            camera.lock().add_component(motion);

        };

        if let Some(light) = light.clone() {
            println!("name: {}", light.lock().name());
            let spinning = Spinning::default();
            light.lock().add_component(spinning);

        };
        renderer.set_camera(camera.unwrap().clone());
        renderer.update_swapchain();
        Ok(Self {
            scene: scene,
            renderer: renderer,
            event_pump: Some(event_loop),
            mouse: RcBox::construct(Mouse::new_with_surface(surface)),
            time: Time::default()
        })
    }

    fn event_loop(mut self) /*-> Arc<EventLoop<()>>*/
    {
        let mut timer = time::Timer::new();
        let mut take_screenshot = false;
        let mut grab_coords = [0i32, 0i32];
        //surface.window().set_cursor_grab(true);
        let event_pump;
        (event_pump, self.event_pump) = (self.event_pump, None);
        
        let event_processor = self.scene.lock().event_processor().clone();

        let app = RcBox::construct(self);
        let app2 = app.clone();

        let mut fps_timer = time::Timer::new();
        event_pump.unwrap().run(move |event: Event<()>, _wtar: &EventLoopWindowTarget<()>, control_flow: &mut ControlFlow| {
            //*control_flow = ControlFlow::Wait;
            let a = app.clone();
            match event {
                Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {event: WindowEvent::Resized(_), ..} => {
                    a.lock().renderer.update_swapchain()
                }
                Event::WindowEvent {event: WindowEvent::CursorMoved{position, ..}, ..} => {
                    a.lock().mouse.lock().cursor_position = [position.x as _, position.y as _]
                }
                Event::DeviceEvent { event, .. } => {
                    match event {
                        DeviceEvent::Key(KeyboardInput { virtual_keycode: Some(VirtualKeyCode::F12), state: ElementState::Pressed, .. }) => {
                            take_screenshot = true;
                        },
                        DeviceEvent::Key(KeyboardInput { virtual_keycode: Some(virtual_keycode), state, ..} ) => {
                            let state = match state {
                                ElementState::Pressed => 1,
                                ElementState::Released => -1,
                            };
                            event_processor.send_event(AbstractEvent::Keyboard(KeyboardEvent{ key_id: virtual_keycode, state }));
                        },
                        DeviceEvent::MouseWheel { delta } => {
                            let (mwdx, mwdy): (i32, i32) = match delta {
                                MouseScrollDelta::LineDelta(x, y) => {
                                    (x as _, y as _)
                                },
                                MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
                                    (x as _, y as _)
                                },
                            };
                            let mwdx = match mwdx {
                                0 => 0,
                                (1..) => 1,
                                _ => -1
                            };
                            let mwdy = match mwdy {
                                0 => 0,
                                (1..) => 1,
                                _ => -1
                            };
                            event_processor.send_event(AbstractEvent::MouseClick(MouseClickEvent{ wheel: (mwdx, mwdy), ..Default::default()}));
                        },
                        DeviceEvent::Button { button, state } => {
                            let dstate = match state {
                                ElementState::Pressed => {
                                    1
                                },
                                ElementState::Released => {
                                    -1
                                },
                            };
                            event_processor.send_event(AbstractEvent::MouseClick(MouseClickEvent{ 
                                lmb: if button==1 {dstate} else {0},
                                mmb: if button==2 {dstate} else {0},
                                rmb: if button==3 {dstate} else {0},
                                ..Default::default()
                            }));
                            match (button, state) {
                                (1, ElementState::Pressed) => {
                                    let slf = a.lock();
                                    let mut mouse = slf.mouse.lock();
                                    grab_coords = mouse.cursor_position;
                                    match mouse.set_cursor_grab(true) {
                                        Ok(_)  => {
                                            drop(mouse.set_cursor_visible(false));
                                        },
                                        Err(_) => ()
                                    };
                                },
                                (1, ElementState::Released) => {
                                    let slf = a.lock();
                                    let mut mouse = slf.mouse.lock();
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
                            a.lock().mouse.lock().mouse_delta = [x as _, y as _];
                            event_processor.send_event(AbstractEvent::MouseMove(MouseMoveEvent{dx: x as _, dy: y as _}));
                        }
                        _ => ()
                    }
                },
                Event::RedrawEventsCleared => {
                    let tu = timer.next_frame();
                    let mut slf = a.lock();
                    slf.time.up_time = tu.uptime as _;
                    slf.time.frame_time = tu.delta as _;
                    slf.renderer.update_timer(tu);
                    slf.renderer.begin_geametry_pass();
                    drop(slf);
                    let scene = app2.lock().scene.clone();
                    scene.lock().step();
                    let objects = a.lock().scene.lock().root_objects();
                    for obj in objects
                    {
                        obj.lock().next_frame();
                        a.lock().renderer.draw(obj.clone());
                    }
                    let event_processor = a.lock().scene.lock().event_processor().clone();
                    let game_logic_thread = std::thread::spawn(move || {
                        event_processor.execute();
                    });
                    if take_screenshot {
                        let mut slf = a.lock();
                        slf.renderer.wait();
                        take_screenshot = false;
                        let img = slf.renderer.postprocessor().get_output("accumulator_out".to_owned()).unwrap().clone();
                        img.save(slf.renderer.queue().clone(), "./screenshot.png");
                    }
                    a.lock().renderer.execute(std::collections::HashMap::new());
                    game_logic_thread.join().unwrap();
                    a.lock().mouse.lock().mouse_delta = [0, 0];

                    let fps_time = fps_timer.next_frame();
                    if fps_time.uptime > 1.0 {
                        println!("fps {}", fps_time.frame);
                        fps_timer = time::Timer::new();
                    };
                },
                _ => { }
            }
        });
    }
}

fn main() {
    let app = App::new("DSGE VK", 960, 720, false, false).unwrap();
    app.event_loop();
}