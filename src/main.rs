extern crate bytemuck;
extern crate byteorder;
extern crate dsge_vk;
extern crate vulkano;
extern crate winit;

use dsge_vk::command_buffer::CommandBufferFather;
use dsge_vk::game_logic::debug_bbox::DisplayOnBboxCorners;
use dsge_vk::game_logic::events::*;
use dsge_vk::game_logic::AbstractEvent;
use dsge_vk::resource_manager::{ResourceManager, ResourceManagerConfig, ResourceManagerRef};
use dsge_vk::scene::{Scene, SceneRef};
use dsge_vk::texture::Texture;
use dsge_vk::*;

use game_logic::motion_example::*;
use game_logic::mouse_look::*;
use vulkano::memory::allocator::StandardMemoryAllocator;

trait Radian {
    fn rad(&self) -> Self;
}

impl Radian for f32 {
    fn rad(&self) -> Self {
        self * 3.1415926535 / 180.0
    }
}

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::SystemTime;

use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::swapchain::Surface;
use vulkano::{Version, VulkanLibrary};
use winit::dpi::PhysicalPosition;
use winit::event::*;
use winit::event::{DeviceEvent, Event, KeyboardInput, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowBuilder};

use references::*;
use renderer::Renderer;

#[derive(Clone)]
pub struct Mouse {
    surface: Arc<Surface>,
    mouse_delta: [i32; 2],
    cursor_position: [i32; 2],
    grabbed: bool,
    visible: bool,
}

impl Mouse {
    fn window(&self) -> &Window
    {
        self.surface.object().unwrap().downcast_ref::<Window>().unwrap()
    }

    fn new_with_surface(surface: Arc<Surface>) -> Self {
        Self {
            surface,
            mouse_delta: [0, 0],
            cursor_position: [0, 0],
            grabbed: false,
            visible: true,
        }
    }

    pub fn set_cursor_position(&self, x: i32, y: i32) -> Result<(), String> {
        let cur_pos = PhysicalPosition::new(x, y);
        match self.window().set_cursor_position(cur_pos) {
            Ok(_) => Ok(()),
            Err(error) => Err(format!("Ошибка установки курсора: {:?}", error)),
        }
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.visible = visible;
        self.window().set_cursor_visible(visible);
    }

    pub fn set_cursor_grab(&mut self, grab: bool) -> Result<(), String> {
        let en_grab = match grab {
            true => CursorGrabMode::Locked,
            false => CursorGrabMode::None,
        };
        match self.window().set_cursor_grab(en_grab) {
            Ok(_) => {
                self.grabbed = grab;
                Ok(())
            }
            Err(error) => Err(format!("Ошибка установки курсора: {:?}", error)),
        }
    }

    #[inline]
    pub fn cursor_grab(&mut self) -> bool {
        self.grabbed
    }

    #[inline]
    pub fn mouse_delta(&self) -> [i32; 2] {
        self.mouse_delta
    }

    #[inline]
    pub fn cursor_position(&self) -> [i32; 2] {
        self.cursor_position
    }
}

pub struct Time {
    frame_time: f64,
    up_time: f64,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            frame_time: 1.0 / 60.0,
            up_time: 0.0,
        }
    }
}

impl Time {
    #[inline]
    pub fn time_delta(&self) -> f64 {
        self.frame_time
    }

    #[inline]
    pub fn up_time(&self) -> f64 {
        self.up_time
    }
}

#[derive(Clone, Copy)]
pub struct AppConfig {
    pub width: u16,
    pub height: u16,
    pub fullscreen: bool,
    pub vsync: bool,
    pub super_resolution: bool,
    pub fxaa: bool,

    resource_manager_config: ResourceManagerConfig,
}

impl AppConfig {
    pub fn resource_manager_config(&self) -> ResourceManagerConfig {
        ResourceManagerConfig {
            super_resolution: self.super_resolution,
            ..self.resource_manager_config
        }
    }

    pub fn resource_manager_config_mut(&mut self) -> &mut ResourceManagerConfig {
        &mut self.resource_manager_config
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            fullscreen: false,
            vsync: false,
            super_resolution: false,
            fxaa: false,

            resource_manager_config: Default::default(),
        }
    }
}

/// Базовый пример приложения
pub struct App {
    resource_manager: ResourceManagerRef,
    renderer: Renderer,
    scene: SceneRef,
    event_pump: Option<EventLoop<()>>,
    mouse: RcBox<Mouse>,
    time: Time,
}

impl App {
    pub fn time(&self) -> &Time {
        &self.time
    }

    pub fn mouse(&self) -> RcBox<Mouse> {
        self.mouse.clone()
    }
}

impl App {
    fn new(title: &str, config: AppConfig, scene_name: String) -> Result<Self, String> {
        let event_loop = EventLoop::new();
        let library = VulkanLibrary::new().unwrap();
        let required_extensions = Surface::required_extensions(&event_loop);
        let vk_instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                max_api_version: Some(Version::major_minor(1, 2)),
                ..Default::default()
            },
        )
        .unwrap();
        // let mut monitors = event_loop.available_monitors();
        // let monitor = monitors.next().expect("no monitor found!");
        // let mut video_modes = monitor.video_modes();
        // video_modes.next();

        let wsize = winit::dpi::PhysicalSize {
            width: config.width,
            height: config.height,
        };
        let window = WindowBuilder::new()
            .with_title(title)
            //.with_fullscreen(if config.fullscreen { Some(Fullscreen::Exclusive(video_modes.next().unwrap())) } else { None } )
            .with_fullscreen(if config.fullscreen {Some(Fullscreen::Borderless(None))} else {None})
            .with_inner_size(wsize)
            .build(&event_loop)
            .unwrap();
        let surface = Surface::from_window(
            vk_instance.clone(), Arc::new(window)
        ).unwrap();

        let (device, mut queues) = Renderer::default_device(vk_instance.clone()).unwrap();
        let queue = queues.next().unwrap();

        // Инициализация менеджера ресурсов
        let resource_manager = ResourceManager::new(
            device.clone(),
            queue.clone(),
            config.resource_manager_config(),
        )?;
        let resource_manager = RcBox::construct(resource_manager);
        // Инициализация рендера
        let mut renderer = Renderer::winit(
            vk_instance,
            resource_manager.clone(),
            surface.clone(),
            [config.width, config.height],
            config.vsync,
            config.super_resolution,
            config.fxaa,
        );

        //let mut renderer = renderer::Renderer::offscreen(vk_instance, [width, height]);
        let (scene, camera) = Scene::from_file(scene_name.as_str(), &mut *resource_manager.lock());
        let objects = scene.lock().root_objects();
        let monkey = objects.iter().find(|obj| obj.lock().name() == "sunh.003");
        let light = objects.iter().find(|obj| obj.lock().name() == "light");

        if let Some(monkey) = monkey {
            println!("name: {}", monkey.lock().name());
            let motion = Spinning::default();
            monkey.lock().add_component(motion);
            monkey.lock().set_static(false);
        };
        //let camera = objects.iter().find(|obj| obj.lock().name()=="Camera");
        if let Some(camera) = camera.clone() {
            //println!("name: {}", camera.lock().name());
            let motion = MouseLook::new(0.001, false);
            camera.lock().add_component(motion);
            camera.lock().set_static(false);

            if let (Some(marker), Some(object)) = (
                objects.iter().find(|obj| obj.lock().name() == "marker"),
                objects.iter().find(|obj| obj.lock().name() == "sunh.003"),
            ) {
                let bbox_dbg = DisplayOnBboxCorners::new(marker.clone());
                object.lock().add_component(bbox_dbg);
            };
        };

        if let Some(light) = light.clone() {
            //println!("name: {}", light.lock().name());
            let spinning = Spinning::default();
            light.lock().add_component(spinning);
        };
        renderer.set_camera(camera.unwrap().clone());
        renderer.update_swapchain(Some([config.width, config.height]));
        Ok(Self {
            scene: scene,
            renderer: renderer,
            resource_manager,
            event_pump: Some(event_loop),
            mouse: RcBox::construct(Mouse::new_with_surface(surface)),
            time: Time::default(),
        })
    }

    fn event_loop(mut self) /*-> Arc<EventLoop<()>>*/
    {
        let queue = self.renderer.queue().clone();
        let command_buffer_father = CommandBufferFather::new(queue.clone());
        let allocator = Arc::new(StandardMemoryAllocator::new_default(queue.device().clone()));
        //let blue_noise = Texture::from_file(self.renderer.queue().clone(), "data/blue_noise_1024.png").unwrap().0;
        let blue_noise = Texture::from_file(
            &command_buffer_father,
            allocator.clone(),
            "data/blue_noise_1024.png",
            false,
            false
        ).unwrap().0;
        /*let blue_noise = Texture::builder()
            .name("blue_noise")
            .read_from_file("data/blue_noise_1024.png")
            .unwrap()
            .pix_fmt(TexturePixelFormat::R8G8B8A8_SNORM)
            .use_case(TextureUseCase::ReadOnly)
            .build(
                &command_buffer_father,
                &allocator,
            )
            .unwrap()
            .0;*/

        let screen_font = Texture::from_file(
            &command_buffer_father,
            allocator.clone(),
            "data/texture/shadertoy_font.png",
            false,
            false
        )
        .unwrap()
        .0;
        let static_input = [
            ("blue_noise".to_owned(), blue_noise),
            ("font".to_owned(), screen_font),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

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
        event_pump.unwrap().run(
            move |event: Event<()>,
                  _wtar: &EventLoopWindowTarget<()>,
                  control_flow: &mut ControlFlow| {
                //*control_flow = ControlFlow::Wait;
                let a = app.clone();
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
                        println!("Изменился размер окна. Меняю разрешение.");
                        a.lock().renderer.update_swapchain(None)
                    }
                    Event::WindowEvent {
                        event: WindowEvent::CursorMoved { position, .. },
                        ..
                    } => a.lock().mouse.lock().cursor_position = [position.x as _, position.y as _],
                    Event::DeviceEvent { event, .. } => match event {
                        DeviceEvent::Key(KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::F11),
                            state: ElementState::Pressed,
                            ..
                        }) => {
                            take_screenshot = true;
                        }
                        DeviceEvent::Key(KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        }) => {
                            control_flow.set_exit();
                        }
                        DeviceEvent::Key(KeyboardInput {
                            virtual_keycode: Some(virtual_keycode),
                            state,
                            ..
                        }) => {
                            let state = match state {
                                ElementState::Pressed => 1,
                                ElementState::Released => -1,
                            };
                            event_processor.send_event(AbstractEvent::Keyboard(KeyboardEvent {
                                key_id: virtual_keycode,
                                state,
                            }));
                        }
                        DeviceEvent::MouseWheel { delta } => {
                            let (mwdx, mwdy): (i32, i32) = match delta {
                                MouseScrollDelta::LineDelta(x, y) => (x as _, y as _),
                                MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
                                    (x as _, y as _)
                                }
                            };
                            let mwdx = match mwdx {
                                0 => 0,
                                (1..) => 1,
                                _ => -1,
                            };
                            let mwdy = match mwdy {
                                0 => 0,
                                (1..) => 1,
                                _ => -1,
                            };
                            event_processor.send_event(AbstractEvent::MouseClick(
                                MouseClickEvent {
                                    wheel: (mwdx, mwdy),
                                    ..Default::default()
                                },
                            ));
                        }
                        DeviceEvent::Button { button, state } => {
                            let dstate = match state {
                                ElementState::Pressed => 1,
                                ElementState::Released => -1,
                            };
                            event_processor.send_event(AbstractEvent::MouseClick(
                                MouseClickEvent {
                                    lmb: if button == 1 { dstate } else { 0 },
                                    mmb: if button == 2 { dstate } else { 0 },
                                    rmb: if button == 3 { dstate } else { 0 },
                                    ..Default::default()
                                },
                            ));
                            match (button, state) {
                                (1, ElementState::Pressed) => {
                                    let slf = a.lock();
                                    let mut mouse = slf.mouse.lock();
                                    grab_coords = mouse.cursor_position;
                                    match mouse.set_cursor_grab(true) {
                                        Ok(_) => {
                                            mouse.set_cursor_visible(false);
                                        }
                                        Err(_) => (),
                                    };
                                }
                                (1, ElementState::Released) => {
                                    let slf = a.lock();
                                    let mut mouse = slf.mouse.lock();
                                    if mouse.cursor_grab() {
                                        match mouse.set_cursor_grab(false) {
                                            Ok(_) => {
                                                drop(mouse.set_cursor_position(
                                                    grab_coords[0],
                                                    grab_coords[1],
                                                ));
                                                mouse.set_cursor_visible(true);
                                            }
                                            Err(_) => (),
                                        };
                                    }
                                }
                                _ => (),
                            }
                        }
                        DeviceEvent::MouseMotion { delta: (x, y) } => {
                            a.lock().mouse.lock().mouse_delta = [x as _, y as _];
                            event_processor.send_event(AbstractEvent::MouseMove(MouseMoveEvent {
                                dx: x as _,
                                dy: y as _,
                            }));
                        }
                        _ => (),
                    },
                    Event::RedrawEventsCleared => {
                        let begin_render_time = SystemTime::now();
                        let tu = timer.next_frame();
                        let mut slf = a.lock();
                        slf.time.up_time = tu.uptime() as _;
                        slf.time.frame_time = tu.delta() as _;
                        slf.renderer.update_timer(tu);
                        slf.renderer.begin_geametry_pass();
                        drop(slf);
                        let scene = app2.lock().scene.clone();
                        scene.lock().step();
                        let objects = a.lock().scene.lock().root_objects();
                        let _begin_render_time =
                            begin_render_time.elapsed().unwrap().as_secs_f64() * 1000.0;
                        let mut _push_to_draw_list_time = 0.0f64;
                        for obj in objects {
                            let t2 = SystemTime::now();
                            a.lock().renderer.draw(obj.clone());
                            let t2 = t2.elapsed().unwrap().as_secs_f64();
                            _push_to_draw_list_time += t2;
                        }
                        _push_to_draw_list_time *= 1000.0;
                        let event_processor = a.lock().scene.lock().event_processor().clone();
                        let game_logic_thread = std::thread::spawn(move || {
                            event_processor.execute();
                        });
                        if take_screenshot {
                            let mut slf = a.lock();
                            slf.renderer.wait();
                            take_screenshot = false;
                            let img = slf
                                .renderer
                                .postprocessor()
                                .get_output("swapchain_out".to_owned())
                                .unwrap()
                                .clone();

                            let fnames = std::fs::read_dir("./screenshots")
                                .unwrap()
                                .map(|fname| {
                                    fname.unwrap().file_name().to_str().unwrap().to_owned()
                                })
                                .collect::<HashSet<_>>();
                            let mut i = 0;
                            let fname = loop {
                                i += 1;
                                let fname = format!("screenshot_{i}.png");
                                if !fnames.contains(&fname) {
                                    break fname;
                                }
                            };
                            img.save(
                                &command_buffer_father,
                                allocator.clone(),
                                format!("./screenshots/{fname}"),
                            )
                            .unwrap();
                        }
                        let _rendering_time = {
                            let rendering_time = SystemTime::now();
                            let mut app = a.lock();
                            let rm = app.resource_manager.clone();
                            app.renderer.execute(&static_input, &rm);
                            let rendering_time =
                                rendering_time.elapsed().unwrap().as_secs_f64() * 1000.0;
                            game_logic_thread.join().unwrap();
                            app.mouse.lock().mouse_delta = [0, 0];
                            rendering_time
                        };

                        let fps_time = fps_timer.next_frame();
                        
                        if fps_time.uptime() > 1.0 {
                            println!("fps {:?}", fps_time.frame());
                            //println!("begin_render_time {begin_render_time:.3} ms");
                            //println!("push_to_draw_list_time {push_to_draw_list_time:.3} ms");
                            //println!("rendering_time {rendering_time:.3} ms");
                            fps_timer = time::Timer::new();
                        };
                    }
                    _ => {}
                }
            },
        );
    }
}

fn main() {
    let mut scene_name = "shooting_range".to_owned();
    let mut config = AppConfig::default();
    let mut args = std::env::args();
    drop(args.next());
    /*let mut super_resolution = false;
    let mut full_screen = false;
    let mut fxaa = false;
    let mut vsync = false;
    let (mut width, mut height) = (640, 360);
    */
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--max-spot-lights" => {
                let li_count = args.next().unwrap();
                if let Ok(li_count) = li_count.parse::<u32>() {
                    config.resource_manager_config_mut().max_spotlights = li_count;
                } else {
                    panic!("Неправильное число для обозначения максимального количества буферов теней для Light::Spot: ({li_count}).");
                }
            }
            "--max-sun-lights" => {
                let li_count = args.next().unwrap();
                if let Ok(li_count) = li_count.parse::<u32>() {
                    config.resource_manager_config_mut().max_sun_lights = li_count;
                } else {
                    panic!("Неправильное число для обозначения максимального количества буферов теней для Light::Spot: ({li_count}).");
                }
            }
            "--max-point-lights" => {
                let li_count = args.next().unwrap();
                if let Ok(li_count) = li_count.parse::<u32>() {
                    config.resource_manager_config_mut().max_point_lights = li_count;
                } else {
                    panic!("Неправильное число для обозначения максимального количества буферов теней для Light::Point: ({li_count}).");
                }
            }
            "--aniso" => {
                let aniso = args.next().unwrap();
                if let Ok(aniso) = aniso.parse::<u32>() {
                    match aniso {
                        0 => (),
                        (1..=16) => config.resource_manager_config_mut().anisotrophy = Some(aniso as f32),
                        _ => panic!("Неправильное число для обозначения степени анизотропной фильтрации {aniso}. Должно быть от 0 до 16.")
                    }
                } else {
                    panic!("Неправильное число для обозначения степени анизотропной фильтрации {aniso}.");
                }
            }
            "--sr" => config.super_resolution = true,
            "--fs" => config.fullscreen = true,
            "--vsync" => config.vsync = true,
            "--fxaa" => config.fxaa = true,
            "--resolution" | "-r" => {
                let _re = args.next().unwrap();
                let mut resol = _re.split("x");
                config.width = str::parse::<u16>(resol.next().unwrap()).unwrap();
                config.height = str::parse::<u16>(resol.next().unwrap()).unwrap();
            }
            _ => scene_name = arg.to_string(),
        }
    }

    let scene = format!("data/scenes/{scene_name}.scene");
    let app = App::new("DSGE VK", config, scene).unwrap();
    app.event_loop();
}
