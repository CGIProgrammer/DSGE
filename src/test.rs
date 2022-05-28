
#[macro_use]
extern crate vulkano;
extern crate vulkano_shaders;
extern crate winit;
extern crate vulkano_win;
extern crate arcball;
extern crate cgmath;
extern crate image;

use vulkano_win::VkSurfaceBuild;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions};
use vulkano::framebuffer::{Framebuffer, Subpass, FramebufferAbstract, RenderPassAbstract};
use vulkano::format::Format;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::image::{SwapchainImage, ImmutableImage, Dimensions};
use vulkano::image::attachment::AttachmentImage;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain;
use vulkano::swapchain::{Swapchain, SurfaceTransform, PresentMode, AcquireError};
use vulkano::sync;
use vulkano::sync::GpuFuture;
use vulkano::sampler::{Sampler, SamplerAddressMode, Filter, MipmapMode};

use winit::{Window, EventsLoop, WindowBuilder};
use image::ImageFormat;
use std::sync::Arc;
use std::iter;

#[derive(Debug, Clone)]
struct Vertex { position: [f32; 3],  tex_coords: [f32; 2]}
impl_vertex!(Vertex, position, tex_coords);

static WINDOW_NAME: &str = "Cube Texture Test";
static WIN_WIDTH: f64 = 700.0;
static WIN_HEIGHT: f64 = 650.0;

struct Vulkan {
    images: Vec<std::sync::Arc<vulkano::image::SwapchainImage<winit::Window>>>,
    swapchain: Arc<vulkano::swapchain::Swapchain<winit::Window>>,
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,
    events_loop: winit::EventsLoop,
    surface: Arc<vulkano::swapchain::Surface<winit::Window>>,
}

impl Vulkan {
    pub fn init_vk() -> Vulkan {
        let extensions = vulkano_win::required_extensions();
        let instance = Instance::new(None, &extensions, None).unwrap();

        let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
        println!("Using device: {} (type: {:?})", physical.name(), physical.ty());

        // events_loop, surface, window
        let  events_loop = EventsLoop::new();
        let surface = WindowBuilder::new()
            .with_dimensions(winit::dpi::LogicalSize {width:WIN_WIDTH, height:WIN_HEIGHT})
            .with_title(WINDOW_NAME.to_string())
            .build_vk_surface(&events_loop, instance.clone()).unwrap();
        
        // (device, queues), queue_family
        let queue_family = physical.queue_families().find(|&q| {
            q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
        }).unwrap();

        let device_ext = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() };
        let (device, mut queues) = Device::new(physical, physical.supported_features(),
            &device_ext, [(queue_family, 0.5)].iter().cloned()).unwrap();
        // we use only one queue, first one
        let queue = queues.next().unwrap();

        let initial_dimensions  = [WIN_WIDTH as u32, WIN_HEIGHT as u32];
        let caps = surface.capabilities(physical).unwrap();
    
        let (swapchain, images) = {
            let usage = caps.supported_usage_flags;
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats[0].0;
            //
            Swapchain::new(device.clone(), surface.clone(), caps.min_image_count, format,
                initial_dimensions, 1, usage, &queue, SurfaceTransform::Identity, alpha,
                PresentMode::Fifo, true, None).unwrap()
        };
      
        Vulkan {
            images,
            swapchain,
            device,
            queue,
            surface,
            events_loop,
        } 
    }
}

pub fn start_test() {
    // Vulkan Object initialization
    let mut vk = Vulkan::init_vk();

    let vertex_buffer = {
        let side2: f32 = 0.8 / 2.0;

        CpuAccessibleBuffer::from_iter(vk.device.clone(), BufferUsage::all(), [
            // Front
            Vertex { position: [-side2, -side2,  side2], tex_coords: [0.0, 0.0] },
            Vertex { position: [ side2, -side2,  side2], tex_coords: [1.0, 0.0] },
            Vertex { position: [ side2,  side2,  side2], tex_coords: [1.0, 1.0] },
            Vertex { position: [-side2,  side2,  side2], tex_coords: [0.0, 1.0] },
    	    // Right
    	    Vertex { position: [ side2, -side2,  side2], tex_coords: [0.0, 0.0] },
    	    Vertex { position: [ side2, -side2, -side2], tex_coords: [1.0, 0.0] },
    	    Vertex { position: [ side2,  side2, -side2], tex_coords: [1.0, 1.0] },
            Vertex { position: [ side2,  side2,  side2], tex_coords: [0.0, 1.0] },
    	    // Back
    	    Vertex { position: [-side2, -side2, -side2], tex_coords: [0.0, 0.0] },
    	    Vertex { position: [-side2,  side2, -side2], tex_coords: [1.0, 0.0] },
    	    Vertex { position: [ side2,  side2, -side2], tex_coords: [1.0, 1.0] },
            Vertex { position: [ side2, -side2, -side2], tex_coords: [0.0, 1.0] },
    	    // Left
    	    Vertex { position: [-side2, -side2,  side2], tex_coords: [0.0, 0.0] },
    	    Vertex { position: [-side2,  side2,  side2], tex_coords: [1.0, 0.0] },
            Vertex { position: [-side2,  side2, -side2], tex_coords: [1.0, 1.0] },
            Vertex { position: [-side2, -side2, -side2], tex_coords: [0.0, 1.0] },
            // Bottom
    	    Vertex { position: [-side2, -side2,  side2], tex_coords: [0.0, 0.0] },
    	    Vertex { position: [-side2, -side2, -side2], tex_coords: [1.0, 0.0] },
    	    Vertex { position: [ side2, -side2, -side2], tex_coords: [1.0, 1.0] },
            Vertex { position: [ side2, -side2,  side2], tex_coords: [0.0, 1.0] },
    	    // Top
            Vertex { position: [-side2,  side2,  side2], tex_coords: [0.0, 0.0] },
    	    Vertex { position: [ side2,  side2,  side2], tex_coords: [1.0, 0.0] },
    	    Vertex { position: [ side2,  side2, -side2], tex_coords: [1.0, 1.0] },
            Vertex { position: [-side2,  side2, -side2], tex_coords: [0.0, 1.0] }
        ].iter().cloned()).expect("failed to create buffer")
    };

    let index_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer
        ::from_iter(vk.device.clone(), vulkano::buffer::BufferUsage::all(), [
            // Front
            0u16, 1, 2, 2, 3, 0,
            // Right
            4, 5, 6, 6, 7, 4,
            // Back
            8, 9, 10, 10, 11, 8,
            // Left
            12, 13, 14, 14, 15, 12,
            // Bottom
            16, 17, 18, 18, 19, 16,
            // Top
            20, 21, 22, 22, 23, 20,
        ].iter().cloned()).expect("failed to create buffer");

    // uniform buffer
    let uniform_buffer = vulkano::buffer::cpu_pool::CpuBufferPool::<vs::ty::Data>
        ::new(vk.device.clone(), vulkano::buffer::BufferUsage::all());

    let vs = vs::Shader::load(vk.device.clone()).expect("failed to create shader module");
    let fs = fs::Shader::load(vk.device.clone()).expect("failed to create shader module");

    // render pass
    let render_pass = Arc::new(single_pass_renderpass!(vk.device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: vk.swapchain.format(),
                samples: 1,
            },
            depth: {
                load: Clear,
                store: DontCare,
                format: vulkano::format::Format::D16Unorm,
                samples: 1,
            } // depth
        },
        pass: {
            color: [color],
            depth_stencil: {depth} // depth
        }
    ).unwrap());

    let window = vk.surface.window();

    let mut dimensions = if let Some(dimensions) = window.get_inner_size() {
        let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
        [dimensions.0, dimensions.1]
    } else {
        return;
    };

    let (texture, tex_future) = {
        let image = image::load_from_memory_with_format(include_bytes!("ume-300x200.png"),
            ImageFormat::PNG).unwrap().to_rgba();
        let image_data = image.into_raw().clone();

        ImmutableImage::from_iter(
            image_data.iter().cloned(),
            Dimensions::Dim2d { width: 300, height: 200 },
            Format::R8G8B8A8Srgb,
            vk.queue.clone()
        ).unwrap()
    };

    let sampler = Sampler::new(vk.device.clone(), Filter::Linear, Filter::Linear,
        MipmapMode::Nearest, SamplerAddressMode::Repeat, SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat, 0.0, 1.0, 0.0, 0.0).unwrap();

    // pipeline and framebuffer
    let (mut pipeline, mut framebuffers) = 
        window_size_dependent_setup(vk.device.clone(), &vs, &fs, &vk.images, render_pass.clone());

    let mut recreate_swapchain = false;
    let mut previous_frame = Box::new(tex_future) as Box<GpuFuture>;

    let persp_proj:cgmath::Matrix4<f32> = cgmath::perspective(cgmath::Deg(65.0),
        dimensions[0] as f32 / dimensions[1] as f32, 0.01, 100.0);
    let mut arcball_camera = {
        let look_at = cgmath::Matrix4::look_at(cgmath::Point3::new(0.0, 0.0, 2.0),
                      cgmath::Point3::new(0.0, 0.0, 0.0), cgmath::Vector3::new(0.0, 1.0, 0.0));
        arcball::ArcballCamera::new(&look_at, 0.05, 4.0, [dimensions[0] as f32,
        dimensions[1] as f32])
    };
    //
    let mut arcball_camera_mat4: [[f32;4];4] = arcball_camera.get_mat4().into();

    let mut mouse_pressed = [false, false];
    let mut prev_mouse: Option<(f64,f64)> = None;

    loop {
        previous_frame.cleanup_finished();

        if recreate_swapchain {
            dimensions = if let Some(dimensions) = window.get_inner_size() {
                let dimensions: (u32, u32) =
                    dimensions.to_physical(window.get_hidpi_factor()).into();
                [dimensions.0, dimensions.1]
            } else {
                return;
            };
            
            let (new_swapchain, new_images) = vk.swapchain.recreate_with_dimension(dimensions)
                .expect("swapcahain not recreate");
            
            vk.swapchain = new_swapchain;

            let (new_pipeline, new_framebuffers) = window_size_dependent_setup(vk.device.clone(), &vs, &fs,
                &new_images, render_pass.clone());
            
            pipeline = new_pipeline;
            framebuffers = new_framebuffers;

            recreate_swapchain = false;
        }

        let proj = (persp_proj * arcball_camera.get_mat4()).into();

        let uniform_buffer_subbuffer = {
            let uniform_data = vs::ty::Data {
                proj : proj,
            };

            uniform_buffer.next(uniform_data).unwrap()
        };

        let set0 = Arc::new(vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
            .add_buffer(uniform_buffer_subbuffer).unwrap()
            .build().unwrap()
        );

        let set1 = Arc::new(vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 1)
            .add_sampled_image(texture.clone(), sampler.clone()).unwrap()
            .build().unwrap()
        );
        
        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(vk.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    recreate_swapchain = true;
                    continue;
                },
                Err(err) => panic!("{:?}", err)
            };

        let command_buffer =
            AutoCommandBufferBuilder::primary_one_time_submit(vk.device.clone(), vk.queue.family()).unwrap() // Ok
                .begin_render_pass(framebuffers[image_num].clone(), false,
                vec![[0.1, 0.1, 0.1, 1.0].into(), 1f32.into()]).unwrap()
            .draw_indexed(pipeline.clone(),
                &DynamicState::none(),
                vec!(vertex_buffer.clone()), index_buffer.clone(),
                (set0.clone(), set1.clone()), ()).unwrap()
            .end_render_pass().unwrap()
            .build().unwrap();

        let _future = previous_frame.join(acquire_future)
            .then_execute(vk.queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(vk.queue.clone(), vk.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();
        
        previous_frame = Box::new(sync::now(vk.device.clone())) as Box<_>;

        let mut done = false;
        vk.events_loop.poll_events(|ev| {
            match ev {
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::CursorMoved { position: winit::dpi::LogicalPosition {x, y}, ..}, ..}
                       if prev_mouse.is_none() => {
                                    prev_mouse = Some((x, y));
                },
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::CursorMoved { position: winit::dpi::LogicalPosition {x, y}, .. }, ..}
                        => {
                            //println!("MouseMoved {},{}", x, y);
                            let prev = prev_mouse.unwrap();
                            if mouse_pressed[0] {
                                arcball_camera.rotate(cgmath::Vector2::new(prev.0 as f32, prev.1 as f32),
                                        cgmath::Vector2::new(x as f32, y as f32));
                                arcball_camera_mat4 = arcball_camera.get_mat4().into();
                                //println!("rotate mat4: {:?}", arcball_camera_mat4);
                            } else if mouse_pressed[1] {
                                let mouse_delta = cgmath::Vector2::new((x - prev.0) as f32, -(y - prev.1) as f32);
                                arcball_camera.pan(mouse_delta, 0.16);
                                arcball_camera_mat4 = arcball_camera.get_mat4().into();
                                //println!("pan mat4: {:?}", arcball_camera_mat4);
                            }
                            prev_mouse = Some((x, y));
                    },
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::MouseInput { state: _state, button: _button, ..}, ..} => {
                        //println!("button {:?}", _button);
                        if _button == winit::MouseButton::Left {
                            mouse_pressed[0] = _state == winit::ElementState::Pressed;
                        } else if _button == winit::MouseButton::Right {
                            mouse_pressed[1] = _state == winit::ElementState::Pressed;
                        }
                },
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::MouseWheel {
                        delta: winit::MouseScrollDelta::LineDelta(_, y), .. }, ..}  => {
                            //println!("ScrollDelta {}", y);
                            arcball_camera.zoom(y, 0.1);
                            arcball_camera_mat4 = arcball_camera.get_mat4().into();
                            //println!("zoom mat4: {:?}", arcball_camera_mat4);
                },
                winit::Event::WindowEvent { event: winit::WindowEvent::CloseRequested, .. } => done = true,
                _ => ()
            }
        });
        if done { return; }
    }
}

fn window_size_dependent_setup(
    device: Arc<Device>,
    vs: &vs::Shader,
    fs: &fs::Shader,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
) -> (Arc<GraphicsPipelineAbstract + Send + Sync>, Vec<Arc<FramebufferAbstract + Send + Sync>> ) {
    let dimensions = images[0].dimensions();

    let depth_buffer = AttachmentImage::transient(device.clone(), dimensions, Format::D16Unorm).unwrap();

    let framebuffers = images.iter().map(|image| {
        Arc::new(
            Framebuffer::start(render_pass.clone())
                .add(image.clone()).unwrap()
                .add(depth_buffer.clone()).unwrap()
                .build().unwrap()
        ) as Arc<FramebufferAbstract + Send + Sync>
    }).collect::<Vec<_>>();

    let pipeline = Arc::new(GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex>()
        .vertex_shader(vs.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .viewports(iter::once(Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0 .. 1.0,
        }))
        .fragment_shader(fs.main_entry_point(), ())
        .depth_stencil_simple_depth()
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap());

    (pipeline, framebuffers)
}

mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: "
            #version 450

            layout(location = 0) in vec3 position;
            layout(location = 1) in vec2 tex_coords;
            layout(location = 0) out vec2 uv;

            layout(set = 0, binding = 0) uniform Data {
                mat4 proj;
            } uniforms;

            void main() 
            {
	            uv = tex_coords;
                gl_Position = uniforms.proj * vec4(position, 1.0);
            }"
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: "
            #version 450

            layout(location = 0) in vec2 uv;
            layout(location = 0) out vec4 f_color;

            layout(set = 1, binding = 0) uniform sampler2D tex;

            void main() 
            {
	            f_color = texture(tex, uv);
            }"
    }
}