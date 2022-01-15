extern crate winit;
extern crate vulkano;
//extern crate nalgebra;

mod teapot;

use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuBufferPool};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents};
use vulkano::device::{
    Device, DeviceExtensions, Features, Queue,
    physical::{
        PhysicalDevice,
        PhysicalDeviceType
    }
};

use vulkano::image::{ImageAccess, ImageUsage, SwapchainImage, AttachmentImage, view::ImageView, ImageCreateFlags, ImageDimensions, ImageLayout, SampleCount, StorageImage};
use vulkano::instance::Instance;
use vulkano::pipeline::{PipelineBindPoint, Pipeline, graphics::viewport::{Viewport}};
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::render_pass::{Framebuffer as VkFramebuffer, RenderPass, RenderPassDesc, AttachmentDesc, SubpassDesc, SubpassDependencyDesc};
use vulkano::swapchain::{self, AcquireError, Swapchain, Surface, SwapchainCreationError, PresentMode};
use vulkano::sync::{self, FlushError, GpuFuture};
use vulkano::format::Format;
use vulkano::Version;
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::window::{Window, WindowBuilder, Fullscreen};

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
use references::*;
use framebuffer::*;
use texture::TexturePixelFormat;
use glenums::GLSLSampler;

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

//#[repr(C)]
#[derive(Default, Debug, Clone)]
struct SmallVert {
    v_pos: [f32; 3],
    v_nor: [f32; 3],
    v_tan: [f32; 3],
    v_bin: [f32; 3],
    v_tex: [f32; 2],
}
vulkano::impl_vertex!(SmallVert, v_pos, v_nor, v_tan, v_bin, v_tex);

/*struct Mesh
{
    _vertex_buffer: Arc<CpuAccessibleBuffer<[SmallVert]>>,
    _index_buffer : Arc<CpuAccessibleBuffer<[u32]>>
}

impl Mesh
{
    pub fn teapot(device: Arc<Device>) -> Self
    {
        let mut mesh = Vec::<SmallVert>::new();
        for i in 0..VERTICES.len() {
            let pos = [VERTICES[i].position.0 / 100.0, VERTICES[i].position.1 / 100.0, VERTICES[i].position.2 / 100.0];
            let nor = [NORMALS[i].normal.0, NORMALS[i].normal.1, NORMALS[i].normal.2];
            let vert = SmallVert{v_pos: pos, v_nor: nor, v_tan: [0.0, 0.0, 0.0], v_bin: [0.0, 0.0, 0.0], v_tex: [0.0, 0.0]};
            mesh.push(vert);
        }
        let vertex_buffer =
            CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, mesh)
                .unwrap();
                
        let indices = INDICES.iter().cloned();
        let index_buffer =
            CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, indices).unwrap();

        Self {
            _vertex_buffer: vertex_buffer,
            _index_buffer: index_buffer
        }
    }
}*/
pub struct Renderer {
    _context: Arc<Instance>,
    _vk_surface: Arc<Surface<Window>>,
    _device: Arc<Device>,
    _default_sc: Arc<Swapchain<Window>>,
    _images: Vec<Arc<SwapchainImage<Window>>>,
    _framebuffers: Vec<FramebufferRef>,
    _render_pass: Arc<RenderPass>,
    _aspect: f32,
    _queue: Arc<Queue>,
    _pipeline: shader::ShaderProgramRef,
    _mesh: MeshRef,
    _texture: TextureRef,
    _previous_frame_end : Option<Box<dyn GpuFuture + 'static>>,
    _distance: f32
}

impl Renderer
{
    pub fn device(&self) -> Arc<Device>
    {
        self._device.clone()
    }

    pub fn swapchain(&self) -> Arc<Swapchain<Window>>
    {
        self._default_sc.clone()
    }

    pub fn images(&self) -> Vec<Arc<SwapchainImage<Window>>>
    {
        self._images.clone()
    }

    pub fn surface(&self) -> Arc<Surface<Window>>
    {
        self._vk_surface.clone()
    }

    pub fn update_swapchain(&mut self)
    {
        let dimensions: [u32; 2] = self._vk_surface.window().inner_size().into();
        let (new_swapchain, new_images) =
            match self._default_sc.recreate().dimensions(dimensions).build() {
                Ok(r) => r,
                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };
        self._aspect = dimensions[0] as f32 / dimensions[1] as f32;
        let db = Texture::new_empty_2d("depth", dimensions[0] as u16, dimensions[1] as u16, TexturePixelFormat::Depth16u, self._queue.clone()).unwrap();
        
        /*let depth_buffer = ImageView::new(
            AttachmentImage::sampled_input_attachment(self._device.clone(), dimensions, Format::D16_UNORM).unwrap(),
        ).unwrap();*/
        
        self._default_sc = new_swapchain;
        let dimensions = new_images[0].dimensions().width_height();

        self._framebuffers = new_images
            .iter()
            .map(|image| {
                let cb = Texture::from_vk_image_view(ImageView::new(image.clone()).unwrap(), self._queue.clone()).unwrap();
                let fb = Framebuffer::new(dimensions[0] as u16, dimensions[1] as u16);
                fb.take_mut().add_color_attachment(cb.clone(), [0.0, 0.0, 0.0, 0.0].into()).unwrap();
                fb.take_mut().set_depth_attachment(db.clone(), 1.0.into());
                fb
            })
            .collect::<Vec<_>>();
    }

    pub fn draw(&mut self) -> bool
    {
        self._previous_frame_end.as_mut().unwrap().cleanup_finished();
        let mut recreate_swapchain = false;
        let (image_num, suboptimal, acquire_future) =
        match swapchain::acquire_next_image(self._default_sc.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                return true;
            }
            Err(e) => panic!("Failed to acquire next image: {:?}", e),
        };
        
        if suboptimal {
            recreate_swapchain = true;
        }

        //let clear_values = vec![[0.5, 0.5, 0.5, 1.0].into(), 1f32.into()];
        //vulkano::command_buffer::AutoCommandBufferBuilder::secondary_graphics(device: Arc<Device>, queue_family: QueueFamily, usage: CommandBufferUsage, subpass: Subpass);

        let x = self._distance.sin();
        let y = self._distance.cos();

        let mut transform = ObjectTransform {
            transform : types::Mat4::new(
                1.0, 0.0, 0.0, x,
                0.0, 1.0, 0.0, y,
                0.0, 0.0, 1.0, -2.0,
                0.0, 0.0, 0.0, 1.0,
            ),
            projection: nalgebra::Perspective3::new(self._aspect, 80.0 * 3.1415926535 / 180.0, 0.1, 100.0).as_matrix().clone()
        };

        
        let subpass = vulkano::render_pass::Subpass::from(self._render_pass.clone(), 0).unwrap();
        let mut pipeline = self._pipeline.take_mut();
        
        pipeline.make_pipeline(subpass.clone());
        pipeline.uniform(&transform.clone(), 0);
        pipeline.uniform(&self._texture.clone(), 1);
        
        self._distance += 0.02;
        transform.transform = types::Mat4::new(
            1.0, 0.0, 0.0,  0.0,
            0.0, 1.0, 0.0,  0.0,
            0.0, 0.0, 1.0, -2.0,
            0.0, 0.0, 0.0,  1.0,
        ) * types::Mat4::new(
             self._distance.cos(), 0.0, self._distance.sin(), 0.0,
             0.0, 1.0, 0.0, 0.0,
            -self._distance.sin(), 0.0, self._distance.cos(), 0.0,
             0.0, 0.0, 0.0, 1.0,
        );
        
        drop(pipeline);
        
        let mut builder = AutoCommandBufferBuilder::primary(
            self._device.clone(),
            self._queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();
        builder
            .bind_framebuffer(self._framebuffers[image_num].clone(), self._render_pass.clone()).unwrap()
            .bind_shader_program(&self._pipeline)
            .bind_shader_uniforms(&self._pipeline)
            .bind_mesh(self._mesh.clone());
        
        self._pipeline.take().uniform(&transform.clone(), 0);
        self._pipeline.take().uniform(&self._texture.clone(), 1);
        
        builder
            .bind_shader_uniforms(&self._pipeline)
            .bind_mesh(self._mesh.clone());
        builder.end_render_pass().unwrap();

        let command_buffer = builder.build().unwrap();

        let future = self._previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self._queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self._queue.clone(), self._default_sc.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self._previous_frame_end = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                recreate_swapchain = true;
                self._previous_frame_end = Some(sync::now(self._device.clone()).boxed());
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                self._previous_frame_end = Some(sync::now(self._device.clone()).boxed());
            }
        };
        return recreate_swapchain;
    }
}

pub struct Application {
    renderer: Renderer,
    event_pump: EventLoop<()>,
    recreate_swapchain: bool
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
            
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };
        let (physical_device, queue_family) = PhysicalDevice::enumerate(&vk_instance)
            .filter(|&p| {
                p.supported_extensions().is_superset_of(&device_extensions)
            })
            .filter_map(|p| {
                p.queue_families()
                    .find(|&q| {
                        q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
                    })
                    .map(|q| (p, q))
            })
            .min_by_key(|(p, _)| {
                match p.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 5,
                    PhysicalDeviceType::IntegratedGpu => 1,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 3,
                    PhysicalDeviceType::Other => 4,
                }
            })
            .unwrap();
        let features = Features {
            sampler_anisotropy: true,
            .. Features::none()
        };
        let (device, mut queues) = Device::new(
            physical_device,
            &features,
            // Some devices require certain extensions to be enabled if they are present
            // (e.g. `khr_portability_subset`). We add them to the device extensions that we're going to
            // enable.
            &physical_device
                .required_extensions()
                .union(&device_extensions),
            [(queue_family, 0.5)].iter().cloned(),
        ).unwrap();
        
        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let queue = queues.next().unwrap();
        let (swapchain, images) = {
            let caps = surface.capabilities(physical_device).unwrap();
            let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
    
            let format = caps.supported_formats[0].0;
            let dimensions: [u32; 2] = [width as u32, height as u32];
            println!("Количество буферов в цепи обновления {}", caps.min_image_count);
            // Please take a look at the docs for the meaning of the parameters we didn't mention.
            Swapchain::start(device.clone(), surface.clone())
                .num_images(caps.min_image_count)
                .format(format)
                .dimensions(dimensions)
                .usage(ImageUsage::color_attachment())
                .present_mode(if _vsync { PresentMode::Fifo } else { PresentMode::Immediate } )
                .sharing_mode(&queue)
                .composite_alpha(composite_alpha)
                .build()
                .unwrap()
        };
        //let mut tb = Texture::builder();
        //tb.anisotropy(16.0).mag_filter(TextureFilter::Linear)
        let texture = Texture::from_file(queue.clone(), "image_img.dds").unwrap();
        texture.take_mut().set_anisotropy(16.0);
        texture.take_mut().set_mipmap(MipmapMode::Linear);
        texture.take_mut().set_mag_filter(TextureFilter::Linear);
        texture.take_mut().update_sampler();

        let mut v_shd = shader::Shader::builder(shader::ShaderType::Vertex, device.clone());
        v_shd
            .default_vertex_attributes()
            .output("col", shader::AttribType::FVec3)
            .output("tex_map", shader::AttribType::FVec2)
            .uniform::<ObjectTransform>("transform", 0)
            //.uniform(&ObjectTransform::default(), "transform_prev", 1)
            .code("
                void main() {
                    col = v_nor;
                    vec4 pos = vec4(v_pos, 1.0);
                    pos = transform.projection * transform.location * pos;
                    tex_map = v_pos.xy*0.5+0.5;
                    gl_Position = pos;
                }"
            ).build().unwrap();

        let mut f_shd = shader::Shader::builder(shader::ShaderType::Fragment, device.clone());
            f_shd
            .input("col", shader::AttribType::FVec3)
            .input("tex_map", shader::AttribType::FVec2)
            .output("f_color", shader::AttribType::FVec4)
            .uniform_sampler2d("tex", 1, false)
            .code("
                void main() {
                    f_color = texture(tex, tex_map.xy);
                    //f_color = vec4(1.0,1.0,1.0, 1.0);
                }"
            ).build().unwrap();
        
        /*let render_pass = vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: Format::D16_UNORM,
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {depth}
                }
            )
            .unwrap();*/
        
        let render_pass_description = RenderPassDesc::new(
            vec![AttachmentDesc {
                format: swapchain.format(),
                samples: SampleCount::Sample1,
                load: vulkano::render_pass::LoadOp::Clear,
                store: vulkano::render_pass::StoreOp::DontCare,
                stencil_load: vulkano::render_pass::LoadOp::DontCare,
                stencil_store: vulkano::render_pass::StoreOp::DontCare,
                initial_layout: ImageLayout::ColorAttachmentOptimal,
                final_layout: ImageLayout::ColorAttachmentOptimal,
            }, AttachmentDesc {
                format: Format::D16_UNORM,
                samples: SampleCount::Sample1,
                load: vulkano::render_pass::LoadOp::Clear,
                store: vulkano::render_pass::StoreOp::DontCare,
                stencil_load: vulkano::render_pass::LoadOp::Clear,
                stencil_store: vulkano::render_pass::StoreOp::Store,
                initial_layout: ImageLayout::DepthStencilAttachmentOptimal,
                final_layout: ImageLayout::DepthStencilAttachmentOptimal,
            }],
            vec![SubpassDesc {
                color_attachments: vec![(0, ImageLayout::ColorAttachmentOptimal)],
                depth_stencil: Some((1, ImageLayout::DepthStencilAttachmentOptimal)),
                input_attachments: vec![],
                resolve_attachments: vec![],
                preserve_attachments: vec![],
            }],
            vec![]
        );
        let render_pass = RenderPass::new(device.clone(), render_pass_description).unwrap();

        //let sas = vs::ty::Data;
        let mut pip_builder = shader::ShaderProgram::builder();
        let graph_pip = pip_builder
            .vertex(&v_shd)
            .fragment(&f_shd)
            .enable_depth_test()
            .build(device.clone()).unwrap();
        
        let mesh = Mesh::builder("default teapot")
            .push_teapot()
            //.push_screen_plane()
            //.push_from_file("../dsge/data/mesh/GiantWarriorChampion_Single.mesh")
            .build(queue.clone()).unwrap();

        //let mesh = Mesh::make_cube("cube", queue.clone()).unwrap();

        Ok(Self {
            renderer: Renderer {
                _context: vk_instance,
                _vk_surface: surface,
                _device: device.clone(),
                _default_sc: swapchain,
                _images: images,
                _render_pass: render_pass,
                _framebuffers: Vec::new(),
                _queue: queue,
                _pipeline: graph_pip,
                _mesh: mesh,
                _texture: texture,
                _previous_frame_end: Some(sync::now(device.clone()).boxed()),
                _distance: 0.0,
                _aspect: 1.0
            },
            event_pump: event_loop,
            recreate_swapchain: false
        })
    }

    pub fn event_loop(mut self) -> Arc<EventLoop<()>>
    {
        let mut frame = 0;
        
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
                    self.recreate_swapchain = true;
                },
                Event::RedrawEventsCleared => {
                    if self.recreate_swapchain {
                        self.renderer.update_swapchain();
                        self.recreate_swapchain = false;
                        println!("recreate_swapchain");
                    }
                    self.recreate_swapchain = self.renderer.draw();
                    //println!("frame {}",frame);
                    frame += 1;
                },
                _ => { }
            }
        });
    }
}

#[derive(Default, Clone)]
struct ObjectTransform
{
    transform: types::Mat4,
    projection: types::Mat4
}
impl shader::ShaderStructUniform for ObjectTransform
{
    fn glsl_type_name() -> String
    {
        String::from("ObjectTransform")
    }

    fn structure() -> String
    {
        String::from("{
            mat4 location;
            mat4 projection;
        }")
    }
    
    fn texture(&self) -> Option<&TextureRef>
    {
        None
    }
}
//impl shader::PipelineUniform for ObjectTransform {}

#[derive(Default)]
struct Location { x: f32, y: f32 }

impl shader::ShaderStructUniform for Location
{
    fn glsl_type_name() -> String
    {
        String::from("Location")
    }

    fn structure() -> String
    {
        String::from("Location")
    }
    
    fn texture(&self) -> Option<&TextureRef>
    {
        None
    }
}

fn main() {
    let mut app = Application::new("DSGE VK", 800, 600, false, true).unwrap();
    //let images = app.images().as_slice();
    //let mut framebuffers = window_size_dependent_setup(&app.images(), render_pass.clone(), &mut viewport);
    app.renderer.update_swapchain();
    app.event_loop();
    println!("Выход");
}