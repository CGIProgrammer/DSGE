extern crate winit;
extern crate vulkano;
//extern crate nalgebra;

mod teapot;

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

use shader::*;
use mesh::*;
use texture::*;

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

pub struct Application {
    renderer: renderer::Renderer,
    event_pump: EventLoop<()>,
    texture: TextureRef,
    shader: ShaderProgramRef,
    mesh: MeshRef,
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

        let renderer = renderer::Renderer::from_winit(vk_instance, surface, _vsync);
        let texture = Texture::from_file(renderer.queue().clone(), "image_img.dds").unwrap();
        texture.take_mut().set_anisotropy(16.0);
        texture.take_mut().set_mipmap(MipmapMode::Linear);
        texture.take_mut().set_mag_filter(TextureFilter::Linear);
        texture.take_mut().update_sampler();

        let mut v_shd = shader::Shader::builder(shader::ShaderType::Vertex, renderer.device().clone());
        v_shd
            .default_vertex_attributes()
            .output("col", shader::AttribType::FVec3)
            .output("nor", shader::AttribType::FVec3)
            .output("tex_map", shader::AttribType::FVec2)
            .uniform::<game_object::GOTransfotmUniform>("object", 0)
            .uniform::<components::camera::CameraUniform>("camera", 0)
            //.uniform(&ObjectTransform::default(), "transform_prev", 1)
            .code("
                void main() {
                    col = v_nor;
                    vec4 pos = camera.projection * object.transform * vec4(v_pos, 1.0);
                    nor = (object.transform * vec4(v_nor, 0.0)).xyz;
                    tex_map = v_pos.xy*0.5+0.5;
                    gl_Position = pos;
                }"
            ).build().unwrap();

        let mut f_shd = shader::Shader::builder(shader::ShaderType::Fragment, renderer.device().clone());
            f_shd
            .input("col", shader::AttribType::FVec3)
            .input("nor", shader::AttribType::FVec3)
            .input("tex_map", shader::AttribType::FVec2)
            .output("gAlbedo", shader::AttribType::FVec3)
            .output("gNormals", shader::AttribType::FVec3)
            .output("gMasks", shader::AttribType::FVec3)
            .output("gVectors", shader::AttribType::FVec4)
            .uniform_sampler2d("tex", 1, false)
            .code("
                void main() {
                    gAlbedo = texture(tex, tex_map.xy).rgb;
                    gNormals = normalize(nor)*0.5+0.5;
                    //f_color = vec4(1.0,1.0,1.0, 1.0);
                }"
            ).build().unwrap();

        let mut pip_builder = shader::ShaderProgram::builder();
        let graph_pip = pip_builder
            .vertex(&v_shd)
            .fragment(&f_shd)
            .enable_depth_test()
            .build(renderer.device().clone()).unwrap();
            
        let mesh = Mesh::builder("default teapot")
            .push_teapot()
            //.push_screen_plane()
            //.push_from_file("../dsge/data/mesh/GiantWarriorChampion_Single.mesh")
            .build(renderer.device().clone()).unwrap();
        
        Ok(Self {
            renderer: renderer,
            event_pump: event_loop,
            shader: graph_pip,
            mesh: mesh,
            texture: texture,
            counter: 0.0
        })
    }

    pub fn event_loop(mut self) -> Arc<EventLoop<()>>
    {
        self.event_pump.run(move |event: Event<()>, _wtar: &EventLoopWindowTarget<()>, control_flow: &mut ControlFlow| {
            //*control_flow = ControlFlow::Wait;
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
                    self.renderer.begin_geametry_pass();
                    self.renderer.draw(self.mesh.clone(), self.texture.clone(), self.shader.clone(), transform);
                    self.renderer.end_frame();
                    self.counter += 0.1;
                },
                _ => { }
            }
        });
    }
}

fn main() {
    let mut app = Application::new("DSGE VK", 800, 600, false, false).unwrap();
    app.renderer.update_swapchain();
    app.event_loop();
    println!("Выход");
}