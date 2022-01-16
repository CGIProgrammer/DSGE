use vulkano::Version;
use vulkano::device::{
    Device, DeviceExtensions, Features, Queue,
    physical::{
        PhysicalDevice,
        PhysicalDeviceType
    }
};
use vulkano::swapchain::{self, AcquireError, Swapchain, SwapchainCreationError, Surface, PresentMode};
use vulkano::image::{view::ImageView, ImageAccess, SwapchainImage, ImageUsage, ImageLayout, SampleCount};
use vulkano::render_pass::{RenderPass, RenderPassDesc, SubpassDesc, AttachmentDesc};
use vulkano::sync::{self, FlushError, GpuFuture};
use vulkano::instance::Instance;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use winit::window::{Window};

use std::sync::Arc;

use crate::game_object::GOTransfotmUniform;
use crate::texture::{Texture, TexturePixelFormat, TextureRef};
use crate::framebuffer::{Framebuffer, FramebufferRef, FramebufferBinder};
use crate::mesh::{MeshRef, Mesh, MeshBinder};
use crate::shader::{ShaderProgramRef, ShaderProgramBinder};
use crate::references::*;
use crate::types::Mat4;
use crate::components::camera::CameraUniform;

struct GBuffer
{
    _device      : Arc<Device>,
    _render_pass : Arc<RenderPass>,
    _albedo      : TextureRef,
    _normals     : TextureRef,
    _specromet   : TextureRef, // specromet - specular, roughness, metallic
    _vectors     : TextureRef,
    _depth       : TextureRef
}

impl GBuffer
{
    fn new(width : u16, height : u16, queue : Arc<Queue>) -> Self
    {
        let formats = [
            TexturePixelFormat::RGB8u,
            TexturePixelFormat::RGB16f,
            TexturePixelFormat::RGB8u,
            TexturePixelFormat::RGBA16f,
            TexturePixelFormat::Depth16u
        ];
        let mut attachments = Vec::new();
        for fmt in formats {
            let img_layout = match fmt {
                TexturePixelFormat::Depth16u |
                TexturePixelFormat::Depth24u |
                TexturePixelFormat::Depth32f => ImageLayout::DepthStencilAttachmentOptimal,
                _ => ImageLayout::ColorAttachmentOptimal
            };
            let att = AttachmentDesc {
                format: fmt.vk_format(),
                samples: SampleCount::Sample1,
                load: vulkano::render_pass::LoadOp::Clear,
                store: vulkano::render_pass::StoreOp::Store,
                stencil_load: vulkano::render_pass::LoadOp::Clear,
                stencil_store: vulkano::render_pass::StoreOp::Store,
                initial_layout: img_layout,
                final_layout: img_layout,
            };
            attachments.push(att);
        }
        let desc = RenderPassDesc::new(
            attachments,
            vec![SubpassDesc {
                color_attachments: vec![
                    (0, ImageLayout::ColorAttachmentOptimal),
                    (1, ImageLayout::ColorAttachmentOptimal),
                    (2, ImageLayout::ColorAttachmentOptimal),
                    (3, ImageLayout::ColorAttachmentOptimal)
                ],
                depth_stencil: Some((4, ImageLayout::DepthStencilAttachmentOptimal)),
                input_attachments: vec![],
                resolve_attachments: vec![],
                preserve_attachments: vec![],
            }],
            vec![]
        );
        Self {
            _device : queue.device().clone(),
            _render_pass : RenderPass::new(queue.device().clone(), desc).unwrap(),
            _albedo : Texture::new_empty_2d("gAlbedo", width, height, TexturePixelFormat::RG8i, queue.clone()).unwrap(),
            _normals : Texture::new_empty_2d("gNormals", width, height, TexturePixelFormat::RG8i, queue.clone()).unwrap(),
            _specromet : Texture::new_empty_2d("gMasks", width, height, TexturePixelFormat::RG8i, queue.clone()).unwrap(),
            _vectors : Texture::new_empty_2d("gVectors", width, height, TexturePixelFormat::RGBA16f, queue.clone()).unwrap(),
            _depth : Texture::new_empty_2d("gDepth", width, height, TexturePixelFormat::Depth16u, queue.clone()).unwrap(),
        }
    }
}

pub struct Renderer
{
    _context : Arc<Instance>,
    _vk_surface : Arc<Surface<Window>>,
    _device : Arc<Device>,
    _queue : Arc<Queue>,
    _swapchain : Arc<Swapchain<Window>>,
    _sc_images : Vec<Arc<SwapchainImage<Window>>>,

    _frame_finish_event : Option<Box<dyn GpuFuture + 'static>>,
    _need_to_update_sc : bool,

    _draw_list : Vec<(MeshRef, TextureRef, ShaderProgramRef, Mat4)>,
    
    _framebuffers : Vec<FramebufferRef>,
    _screen_plane : MeshRef,
    _gbuffer : GBuffer,
    _postprocess_pass : Arc<RenderPass>,
    _aspect : f32,
    _camera : CameraUniform
}

#[allow(dead_code)]
impl Renderer
{
    pub fn from_winit(vk_instance : Arc<Instance>, win: Arc<Surface<Window>>, vsync: bool) -> Self
    {
        let dimensions = win.window().inner_size().into();
        println!("{:?}", dimensions);
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
                        q.supports_graphics() && win.is_supported(q).unwrap_or(false)
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
        let (swapchain, images) : (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) = {
            let caps = win.capabilities(physical_device).unwrap();
            let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
    
            let format = caps.supported_formats[0].0;
            
            //println!("Количество буферов в цепи обновления {}", caps.min_image_count);
            Swapchain::start(device.clone(), win.clone())
                .num_images(caps.min_image_count)
                .format(format)
                .dimensions(dimensions)
                .usage(ImageUsage::color_attachment())
                .present_mode(if vsync { PresentMode::Fifo } else { PresentMode::Immediate } )
                .sharing_mode(&queue)
                .composite_alpha(composite_alpha)
                .build()
                .unwrap()
        };

        let final_renderpass = RenderPassDesc::new(
            vec![
            AttachmentDesc {
                format: swapchain.format(),
                samples: SampleCount::Sample1,
                load: vulkano::render_pass::LoadOp::Clear,
                store: vulkano::render_pass::StoreOp::Store,
                stencil_load: vulkano::render_pass::LoadOp::Clear,
                stencil_store: vulkano::render_pass::StoreOp::Store,
                initial_layout: ImageLayout::ColorAttachmentOptimal,
                final_layout: ImageLayout::ColorAttachmentOptimal,
            },
            AttachmentDesc {
                format: TexturePixelFormat::Depth16u.vk_format(),
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
        let final_render_pass = RenderPass::new(device.clone(), final_renderpass).unwrap();
        
        Renderer {
            _vk_surface : win,
            _swapchain : swapchain,
            _aspect : dimensions[0] as f32 / dimensions[1] as f32,
            _context : vk_instance,
            _device : device.clone(),
            _queue : queue.clone(),
            _postprocess_pass : final_render_pass,
            _sc_images : images,
            _framebuffers : Vec::new(),
            _screen_plane : Mesh::make_screen_plane(queue.clone()).unwrap(),
            _gbuffer : GBuffer::new(dimensions[0] as u16, dimensions[1] as u16, queue.clone()),
            _need_to_update_sc : true,
            _frame_finish_event : Some(sync::now(device.clone()).boxed()),
            _draw_list : Vec::new(),
            _camera : CameraUniform::identity(dimensions[0] as f32 / dimensions[1] as f32, 80.0, 0.1, 100.0)
        }
    }

    pub fn width(&self) -> u16
    {
        self._vk_surface.window().inner_size().width as u16
    }

    pub fn height(&self) -> u16
    {
        self._vk_surface.window().inner_size().height as u16
    }

    pub fn update_swapchain(&mut self)
    {
        let dimensions: [u32; 2] = self._vk_surface.window().inner_size().into();
        let (new_swapchain, new_images) =
            match self._swapchain.recreate().dimensions(dimensions).build() {
                Ok(r) => r,
                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };
        self._aspect = dimensions[0] as f32 / dimensions[1] as f32;
        let db = Texture::new_empty_2d("depth", dimensions[0] as u16, dimensions[1] as u16, TexturePixelFormat::Depth16u, self._queue.clone()).unwrap();
        
        self._swapchain = new_swapchain;
        let dimensions = new_images[0].dimensions().width_height();

        self._framebuffers = new_images
            .iter()
            .map(|image| {
                let cb = Texture::from_vk_image_view(ImageView::new(image.clone()).unwrap(), self._queue.clone()).unwrap();
                let fb = Framebuffer::new(dimensions[0] as u16, dimensions[1] as u16);
                fb.take_mut().add_color_attachment(cb.clone(), [0.0, 0.0, 0.0, 1.0].into()).unwrap();
                fb.take_mut().set_depth_attachment(db.clone(), 1.0.into());
                fb
            })
            .collect::<Vec<_>>();
    }

    pub fn draw(&mut self, mesh : MeshRef, tex : TextureRef, shader : ShaderProgramRef, transform : Mat4)
    {
        self._draw_list.push((mesh, tex, shader, transform));
    }

    pub fn begin_frame(&mut self)
    {
        self._draw_list.clear();
    }

    pub fn end_frame(&mut self)
    {
        self._frame_finish_event.as_mut().unwrap().cleanup_finished();
        if self._need_to_update_sc {
            self.update_swapchain();
            self._need_to_update_sc = false;
        }
        let (image_num, suboptimal, acquire_future) =
        match swapchain::acquire_next_image(self._swapchain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                self.update_swapchain();
                self.begin_frame();
                return;
            }
            Err(e) => panic!("Failed to acquire next image: {:?}", e),
        };
        
        if suboptimal {
            self._need_to_update_sc = true;
        }

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self._device.clone(),
            self._queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();
        command_buffer_builder.bind_framebuffer(self._framebuffers[image_num].clone(), self._postprocess_pass.clone()).unwrap();

        let mut shid = -1;
        let mut meid = -1;
        for (mesh, texture, shader, transform) in &self._draw_list
        {
            if shader.box_id() != shid {
                shid = shader.box_id();
                meid = -1;
                shader.take_mut().make_pipeline(vulkano::render_pass::Subpass::from(self._postprocess_pass.clone(), 0).unwrap());
                command_buffer_builder
                    .bind_shader_program(shader);
            }
            let tr = GOTransfotmUniform {
                transform : transform.clone(),
                transform_prev : transform.clone()
            };
            let mut shd = shader.take_mut();
            shd.uniform(&tr, 0);
            shd.uniform(&self._camera, 0);
            shd.uniform(texture, 1);
            drop(shd);
            command_buffer_builder.bind_shader_uniforms(shader);
            
            if mesh.box_id() != meid {
                command_buffer_builder.bind_mesh(mesh.clone());
                meid = mesh.box_id();
            }
        }
        command_buffer_builder.end_render_pass().unwrap();
        let command_buffer = command_buffer_builder.build().unwrap();
        let future = self._frame_finish_event
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self._queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self._queue.clone(), self._swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self._frame_finish_event = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                self._need_to_update_sc = true;
                self._frame_finish_event = Some(sync::now(self._device.clone()).boxed());
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                self._frame_finish_event = Some(sync::now(self._device.clone()).boxed());
            }
        };
    }

    pub fn queue(&self) -> &Arc<Queue>
    {
        &self._queue
    }

    pub fn device(&self) -> &Arc<Device>
    {
        &self._device
    }
}