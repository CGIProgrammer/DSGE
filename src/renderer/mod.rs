pub mod postprocessor;
use vulkano::device::{
    Device, DeviceExtensions, Features, Queue,
    physical::{
        PhysicalDevice,
        PhysicalDeviceType
    }
};
use vulkano::swapchain::{self, AcquireError, Swapchain, SwapchainCreationError, Surface, PresentMode};
use vulkano::image::{view::ImageView, ImageAccess, SwapchainImage, ImageUsage, ImageLayout, SampleCount};
use vulkano::render_pass::{RenderPass, RenderPassDesc, Subpass, SubpassDesc, AttachmentDesc};
use vulkano::sync::{self, FlushError, GpuFuture};
use vulkano::instance::Instance;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer};
use winit::window::{Window};

use std::sync::Arc;

use crate::game_object::GOTransfotmUniform;
use crate::texture::{Texture, TexturePixelFormat, TextureRef};
use crate::framebuffer::{Framebuffer, FramebufferRef, FramebufferBinder};
use crate::mesh::{MeshRef, Mesh, MeshBinder};
use crate::shader::{ShaderProgramRef, ShaderStructUniform, ShaderProgramBinder, Shader, ShaderProgram, ShaderType};
use crate::references::*;
use crate::types::Mat4;
use crate::glenums::AttribType;
use crate::components::camera::CameraUniform;
use postprocessor::RenderPostprocessingGraph;

impl ShaderStructUniform for i32
{
    fn structure() -> String
    {
        "{int value;}".to_string()
    }

    fn glsl_type_name() -> String
    {
        "32DummyInt".to_string()
    }

    fn texture(&self) -> Option<&TextureRef>
    {
        None
    }
}

/*
 * Буфер для сохранения результатов прохода геометрии (geometry pass)
 */
struct GBuffer
{
    _frame_buffer: FramebufferRef,
    _device      : Arc<Device>,
    _geometry_pass : Arc<RenderPass>,
    _albedo      : TextureRef, // Цвет поверхности
    _normals     : TextureRef, // Нормали
    _specromet   : TextureRef, // specromet - specular, roughness, metallic. TODO пока ничем не заполняется
    _vectors     : TextureRef, // Векторы скорости. TODO пока ничем не заполняется
    _depth       : TextureRef  // Глубина. TODO пока ничем не заполняется
}

impl GBuffer
{
    fn new(width : u16, height : u16, device : Arc<Device>) -> Self
    {
        let fb = Framebuffer::new(width, height);
        let mut _fb = fb.take_mut();
        let albedo = Texture::new_empty_2d("gAlbedo", width, height, TexturePixelFormat::RGBA8i, device.clone()).unwrap();
        let normals = Texture::new_empty_2d("gNormals", width, height, TexturePixelFormat::RGBA8i, device.clone()).unwrap();
        let masks = Texture::new_empty_2d("gMasks", width, height, TexturePixelFormat::RGBA8i, device.clone()).unwrap();
        let vectors = Texture::new_empty_2d("gVectors", width, height, TexturePixelFormat::RGBA16f, device.clone()).unwrap();
        let depth = Texture::new_empty_2d("gDepth", width, height, TexturePixelFormat::Depth16u, device.clone()).unwrap();
        _fb.add_color_attachment(albedo.clone(), [0.5, 0.5, 0.5].into()).unwrap();
        _fb.add_color_attachment(normals.clone(), [0.0, 0.0, 0.0].into()).unwrap();
        _fb.add_color_attachment(masks.clone(), [0.0, 0.0, 0.0].into()).unwrap();
        _fb.add_color_attachment(vectors.clone(), [0.0, 0.0, 0.0, 0.0].into()).unwrap();
        _fb.set_depth_attachment(depth.clone(), 1.0.into());
        drop(_fb);

        let formats = [
            TexturePixelFormat::RGBA8i,
            TexturePixelFormat::RGBA8i,
            TexturePixelFormat::RGBA8i,
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
            _device : device.clone(),
            _geometry_pass : RenderPass::new(device.clone(), desc).unwrap(),
            _albedo : albedo,
            _normals : normals,
            _specromet : masks,
            _vectors : vectors,
            _depth : depth,
            _frame_buffer : fb
        }
    }
}

/// Основная структура для рендеринга
pub struct Renderer
{
    _context : Arc<Instance>,
    _vk_surface : Arc<Surface<Window>>,
    _device : Arc<Device>,
    _queue : Arc<Queue>,
    _swapchain : Arc<Swapchain<Window>>,
    _sc_images : Vec<Arc<SwapchainImage<Window>>>,
    _sc_textures : Vec<TextureRef>,

    _frame_finish_event : Option<Box<dyn GpuFuture + 'static>>,
    _need_to_update_sc : bool,

    _draw_list : Vec<(MeshRef, TextureRef, ShaderProgramRef, Mat4)>,
    
    _framebuffers : Vec<FramebufferRef>,
    _screen_plane : MeshRef,
    _screen_plane_shader : ShaderProgramRef,
    _gbuffer : GBuffer,
    _postprocess_pass : Arc<RenderPass>,
    _aspect : f32,
    _camera : CameraUniform,

    _postprocessor : RenderPostprocessingGraph,
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
            &physical_device
                .required_extensions()
                .union(&device_extensions),
            [(queue_family, 0.5)].iter().cloned(),
        ).unwrap();
        
        println!(
            "Используется устройство: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let queue = queues.next().unwrap();
        let (swapchain, images) : (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) = {
            let caps = win.capabilities(physical_device).unwrap();
            let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
    
            let format = caps.supported_formats[0].0;
            
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
                load: vulkano::render_pass::LoadOp::DontCare,
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
        
        let mut result = Renderer {
            _vk_surface : win,
            _swapchain : swapchain,
            _aspect : dimensions[0] as f32 / dimensions[1] as f32,
            _context : vk_instance,
            _device : device.clone(),
            _queue : queue.clone(),
            _postprocess_pass : final_render_pass,
            _sc_images : images,
            _framebuffers : Vec::new(),
            _screen_plane : Mesh::make_screen_plane(device.clone()).unwrap(),
            _screen_plane_shader : Self::make_screen_plane_shader(device.clone()),
            _gbuffer : GBuffer::new(dimensions[0] as u16, dimensions[1] as u16, device.clone()),
            _need_to_update_sc : true,
            _frame_finish_event : Some(sync::now(device.clone()).boxed()),
            _draw_list : Vec::new(),
            _camera : CameraUniform::identity(dimensions[0] as f32 / dimensions[1] as f32, 80.0, 0.1, 100.0),
            _sc_textures : Vec::new(),
            _postprocessor : RenderPostprocessingGraph::new(queue.clone(), dimensions[0] as u16, dimensions[1] as u16)
        };
        result.resize(dimensions[0] as u16, dimensions[1] as u16);
        result
    }

    pub fn resize(&mut self, width: u16, height: u16)
    {
        self._gbuffer = GBuffer::new(width, height, self._device.clone());
        /* Создание узлов и связей графа постобработки */
        /* На данный момент это размытие в движении */
        self._postprocessor.reset();
        let acc = self._postprocessor.acc_mblur(width, height);  // Создание ноды размытия в движении
        self._postprocessor.link_stages(acc, 0, acc, 2);  // Соединение накопительного выхода со входом ноды
        self._postprocessor.link_stages(acc, 1, 0, 0);    // Соединение ноды с выходом.
    }

    pub fn width(&self) -> u16
    {
        self._vk_surface.window().inner_size().width as u16
    }

    pub fn height(&self) -> u16
    {
        self._vk_surface.window().inner_size().height as u16
    }

    /// Обновление swapchain изображений
    /// Как правило необходимо при изменении размера окна
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
        let db = Texture::new_empty_2d("depth", dimensions[0] as u16, dimensions[1] as u16, TexturePixelFormat::Depth16u, self._device.clone()).unwrap();
        
        self._swapchain = new_swapchain;
        let dimensions = new_images[0].dimensions().width_height();

        self._framebuffers.clear();
        self._sc_textures.clear();
        for image in new_images
        {
            let cb = Texture::from_vk_image_view(ImageView::new(image.clone()).unwrap(), self._device.clone()).unwrap();
            let fb = Framebuffer::new(dimensions[0] as u16, dimensions[1] as u16);
            fb.take_mut().add_color_attachment(cb.clone(), [0.0, 0.0, 0.0, 1.0].into()).unwrap();
            fb.take_mut().set_depth_attachment(db.clone(), 1.0.into());
            self._framebuffers.push(fb);
            self._sc_textures.push(cb);
        }
        self.resize(dimensions[0] as u16, dimensions[1] as u16);
    }

    /// Начинает проход геометрии
    pub fn begin_geametry_pass(&mut self)
    {
        self._draw_list.clear();
    }

    /// Передаёт объект для растеризации
    pub fn draw(&mut self, mesh : MeshRef, tex : TextureRef, shader : ShaderProgramRef, transform : Mat4)
    {
        self._draw_list.push((mesh, tex, shader, transform));
    }

    /// Фомирует буфер команд GPU
    pub fn build_geametry_pass(&self) -> PrimaryAutoCommandBuffer
    {
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self._device.clone(),
            self._queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();
        let gbuffer_rp = &self._gbuffer._geometry_pass;
        let gbuffer_fb = &self._gbuffer._frame_buffer;
        let geom_pass = Subpass::from(gbuffer_rp.clone(), 0).unwrap();

        command_buffer_builder
            .bind_framebuffer(gbuffer_fb.clone(), gbuffer_rp.clone()).unwrap();

        let mut shid = -1;
        for (mesh, texture, shader, transform) in &self._draw_list
        {
            if shader.box_id() != shid {
                shid = shader.box_id();
                shader.take_mut().make_pipeline(geom_pass.clone());
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
            
            command_buffer_builder.bind_mesh(mesh);
        }

        command_buffer_builder.end_render_pass().unwrap();
        command_buffer_builder.build().unwrap()
    }

    /// Тестовая функция постобработки. Просто пропускает изображение через шейдер.
    fn postprocess_pass(&self, sci_index : usize) -> PrimaryAutoCommandBuffer
    {
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self._device.clone(),
            self._queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();
        let subpass = vulkano::render_pass::Subpass::from(self._postprocess_pass.clone(), 0).unwrap();
        
        let mut shd = self._screen_plane_shader.take_mut();
        shd.make_pipeline(subpass);
        shd.uniform(&self._gbuffer._albedo, 1);
        drop(shd);

        command_buffer_builder
            .bind_framebuffer(self._framebuffers[sci_index].clone(), self._postprocess_pass.clone()).unwrap()
            .bind_shader_program(&self._screen_plane_shader)
            .bind_shader_uniforms(&self._screen_plane_shader)
            .bind_mesh(&self._screen_plane);
        
        command_buffer_builder.end_render_pass().unwrap();
        command_buffer_builder.build().unwrap()
    }

    /// Выполняет все сформированные буферы команд
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
                self.begin_geametry_pass();
                return;
            }
            Err(e) => panic!("Failed to acquire next image: {:?}", e),
        };
        
        if suboptimal {
            self._need_to_update_sc = true;
        }
        
        self._postprocessor.set_input(1, 1, &self._gbuffer._albedo);
        self._postprocessor.set_output(0, self._sc_textures[image_num].clone());

        let pp_command_buffer = self._postprocessor.execute_graph();
        let gp_command_buffer = self.build_geametry_pass();

        //let mut f2 : Option<vulkano::command_buffer::CommandBufferExecFuture<_, _>>;
        let future = self._frame_finish_event
            .take().unwrap()
            .then_execute(self._queue.clone(), gp_command_buffer).unwrap()
            .join(acquire_future)
            .then_execute_same_queue(pp_command_buffer).unwrap()
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

    /// Создаёт шейдер для тестовой потобработки.
    fn make_screen_plane_shader(device: Arc<Device>) -> ShaderProgramRef
    {
        let mut v_builder = Shader::builder(ShaderType::Vertex, device.clone());
        let mut f_builder = Shader::builder(ShaderType::Fragment, device.clone());
        v_builder
            .default_vertex_attributes()
            .output("coords", AttribType::FVec2)
            .code("
                void main() {
                    coords = v_pos.xy*0.5+0.5;
                    gl_Position = vec4(v_pos, 1.0);
                }
            ");
        f_builder
            .input("coords", AttribType::FVec2)
            .uniform_sampler2d("img", 1, false)
            //.uniform_sampler2d("prev", 1, false)
            .output("fragColor", AttribType::FVec4)
            .code("
                void main() {
                    //vec4 a = texture(prev, coords);
                    vec4 b = texture(img, coords);
                    fragColor = b; //mix(a, b, 0.1);
                }
            ")
        ;
        let mut sh_builder = ShaderProgram::builder();
        sh_builder.vertex(&v_builder.build().unwrap());
        sh_builder.fragment(&f_builder.build().unwrap());
        sh_builder.build(device.clone()).unwrap()
    }
}