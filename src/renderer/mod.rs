pub mod postprocessor;
use vulkano::device::{
    Device, DeviceExtensions, Features, Queue, DeviceCreateInfo, QueueCreateInfo,
    physical::{
        PhysicalDevice,
        PhysicalDeviceType
    }
};
use vulkano::swapchain::{self, AcquireError, Swapchain, Surface, PresentMode, SwapchainCreateInfo};
use vulkano::image::{view::ImageView, SwapchainImage, ImageUsage, ImageLayout, SampleCount};
use vulkano::render_pass::{RenderPass, RenderPassCreateInfo, SubpassDescription, AttachmentDescription, AttachmentReference};
use vulkano::sync::{self, FlushError, GpuFuture};
use vulkano::instance::Instance;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer};
use winit::window::{Window};

use std::sync::{Arc, LockResult};

use crate::texture::{Texture, TexturePixelFormat, TextureDimensions, TextureRef};
use crate::framebuffer::{Framebuffer, FramebufferRef, FramebufferBinder};
use crate::mesh::{MeshRef, Mesh};
use crate::shader::ShaderStructUniform;
use crate::references::*;
use crate::game_object::*;
use postprocessor::Postprocessor;
use crate::time::UniformTime;

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
    fn new(width : u16, height : u16, queue : Arc<Queue>) -> Self
    {
        println!("Создание нового G-буфера {}x{}", width, height);
        let device = queue.device();
        let formats = [
            TexturePixelFormat::RGBA8u,
            TexturePixelFormat::RGBA8u,
            TexturePixelFormat::RGBA8u,
            TexturePixelFormat::RGBA16f,
            TexturePixelFormat::Depth16u
        ];

        let fb = Framebuffer::new(width, height);
        let mut _fb = fb.take_mut();
        let dims = TextureDimensions::Dim2d{
            width: width as _,
            height: height as _,
            array_layers: 1
        };
        let albedo = Texture::new_empty_mutex("gAlbedo", dims, formats[0], device.clone()).unwrap();
        let normals = Texture::new_empty_mutex("gNormals", dims, formats[1], device.clone()).unwrap();
        let masks = Texture::new_empty_mutex("gMasks", dims, formats[2], device.clone()).unwrap();
        let vectors = Texture::new_empty_mutex("gVectors", dims, formats[3], device.clone()).unwrap();
        let depth = Texture::new_empty_mutex("gDepth", dims, formats[4], device.clone()).unwrap();

        albedo.take().clear_color(queue.clone());
        albedo.take().set_horizontal_address(crate::texture::TextureRepeatMode::ClampToEdge);
        albedo.take().set_vertical_address(crate::texture::TextureRepeatMode::ClampToEdge);
        albedo.take().update_sampler();
        normals.take().clear_color(queue.clone());
        masks.take().clear_color(queue.clone());
        vectors.take().clear_color(queue.clone());

        _fb.add_color_attachment(albedo.clone(), [0.0, 0.0, 0.0, 0.0].into()).unwrap();
        _fb.add_color_attachment(normals.clone(), [0.0, 0.0, 0.0, 0.0].into()).unwrap();
        _fb.add_color_attachment(masks.clone(), [0.0, 0.0, 0.0, 0.0].into()).unwrap();
        _fb.add_color_attachment(vectors.clone(), [0.0, 0.0, 0.0, 0.0].into()).unwrap();
        _fb.set_depth_attachment(depth.clone(), 1.0.into());
        drop(_fb);

        let mut attachments = Vec::new();
        for fmt in formats {
            let img_layout = match fmt {
                TexturePixelFormat::Depth16u |
                TexturePixelFormat::Depth24u |
                TexturePixelFormat::Depth32f => ImageLayout::DepthStencilAttachmentOptimal,
                _ => ImageLayout::ColorAttachmentOptimal
            };
            let att = AttachmentDescription {
                format: Some(fmt.vk_format()),
                samples: SampleCount::Sample1,
                load_op: vulkano::render_pass::LoadOp::Clear,
                store_op: vulkano::render_pass::StoreOp::Store,
                stencil_load_op: vulkano::render_pass::LoadOp::Clear,
                stencil_store_op: vulkano::render_pass::StoreOp::Store,
                initial_layout: img_layout,
                final_layout: img_layout,
                ..Default::default()
            };
            attachments.push(att);
        }
        let desc = RenderPassCreateInfo {
            attachments: attachments,
            subpasses: vec![SubpassDescription {
                color_attachments: vec![
                    Some(AttachmentReference{attachment: 0, layout: ImageLayout::ColorAttachmentOptimal, ..Default::default()}),
                    Some(AttachmentReference{attachment: 1, layout: ImageLayout::ColorAttachmentOptimal, ..Default::default()}),
                    Some(AttachmentReference{attachment: 2, layout: ImageLayout::ColorAttachmentOptimal, ..Default::default()}),
                    Some(AttachmentReference{attachment: 3, layout: ImageLayout::ColorAttachmentOptimal, ..Default::default()})
                ],
                depth_stencil_attachment: Some(AttachmentReference{attachment: 4, layout: ImageLayout::DepthStencilAttachmentOptimal, ..Default::default()}),
                ..Default::default()
            }],
            ..Default::default()
        };

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

    _draw_list : Vec<RcBox<dyn GameObject>>,
    
    _framebuffers : Vec<FramebufferRef>,
    _screen_plane : MeshRef,
    //_screen_plane_shader : ShaderProgramRef,
    _gbuffer : GBuffer,
    _postprocess_pass : Arc<RenderPass>,
    _aspect : f32,
    _camera : Option<RcBox<dyn GameObject>>,

    _postprocessor : Postprocessor,
    _timer : UniformTime
}

#[allow(dead_code)]
impl Renderer
{
    pub fn postprocessor(&mut self) -> &mut Postprocessor
    {
        &mut self._postprocessor
    }
}

#[allow(dead_code)]
impl Renderer
{
    pub fn from_winit(vk_instance : Arc<Instance>, win: Arc<Surface<Window>>, vsync: bool) -> Self
    {
        let dimensions: [f32; 2] = win.window().inner_size().into();
        println!("{:?}", dimensions);
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            //ext_extended_dynamic_state: true,
            ..DeviceExtensions::none()
        };
        let (physical_device, queue_family) = PhysicalDevice::enumerate(&vk_instance)
            .filter(|&p| {
                p.supported_extensions().is_superset_of(&device_extensions)
            })
            .filter_map(|p| {
                p.queue_families()
                    .find(|&q| {
                        q.supports_graphics() && q.supports_surface(&win).unwrap_or(false)
                    })
                    .map(|q| (p, q))
            })
            .min_by_key(|(p, _)| {
                match p.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 1,
                    PhysicalDeviceType::IntegratedGpu => 0,
                    PhysicalDeviceType::VirtualGpu => 3,
                    PhysicalDeviceType::Cpu => 4,
                    PhysicalDeviceType::Other => 5,
                }
            })
            .unwrap();
        let features = Features {
            sampler_anisotropy: true,
            //extended_dynamic_state: true,
            .. Features::none()
        };
        let dev_info = DeviceCreateInfo {
            enabled_features: features,
            queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
            enabled_extensions: physical_device
                .required_extensions()
                .union(&device_extensions),
            ..Default::default()
        };
        let (device, mut queues) = Device::new(physical_device, dev_info).unwrap();
        
        println!(
            "Используется устройство: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let queue = queues.next().unwrap();
        let (swapchain, images) : (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) = {
            let caps = physical_device.surface_capabilities(&win, Default::default()).unwrap();
            let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
    
            let format = Some(
                physical_device
                    .surface_formats(&win, Default::default())
                    .unwrap()[0]
                    .0,
            );
            
            Swapchain::new(device.clone(), win.clone(), SwapchainCreateInfo{
                min_image_count: caps.min_image_count,
                image_format: format,
                image_extent: win.window().inner_size().into(),
                image_usage: ImageUsage::color_attachment(),
                composite_alpha: composite_alpha,
                present_mode: if vsync { PresentMode::Fifo } else { PresentMode::Immediate },
                ..Default::default()
            }).unwrap()
        };

        let final_renderpass = RenderPassCreateInfo{
            attachments: vec![
            AttachmentDescription {
                format: Some(swapchain.image_format()),
                samples: SampleCount::Sample1,
                load_op: vulkano::render_pass::LoadOp::Clear,
                store_op: vulkano::render_pass::StoreOp::Store,
                stencil_load_op: vulkano::render_pass::LoadOp::DontCare,
                stencil_store_op: vulkano::render_pass::StoreOp::DontCare,
                final_layout: ImageLayout::ColorAttachmentOptimal,
                ..Default::default()
            },
            AttachmentDescription {
                format: Some(TexturePixelFormat::Depth16u.vk_format()),
                samples: SampleCount::Sample1,
                load_op: vulkano::render_pass::LoadOp::Clear,
                store_op: vulkano::render_pass::StoreOp::Store,
                stencil_load_op: vulkano::render_pass::LoadOp::Clear,
                stencil_store_op: vulkano::render_pass::StoreOp::Store,
                initial_layout: ImageLayout::DepthStencilAttachmentOptimal,
                final_layout: ImageLayout::DepthStencilAttachmentOptimal,
                ..Default::default()
            }],
            subpasses: vec![SubpassDescription {
                color_attachments: vec![Some(AttachmentReference{attachment: 0, layout: ImageLayout::ColorAttachmentOptimal, ..Default::default()})],
                depth_stencil_attachment: Some(AttachmentReference{attachment: 1, layout: ImageLayout::DepthStencilAttachmentOptimal, ..Default::default()}),
                ..Default::default()
            }],
            ..Default::default()
        };
        let final_render_pass = RenderPass::new(device.clone(), final_renderpass).unwrap();

        let result = Renderer {
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
            _gbuffer : GBuffer::new(dimensions[0] as u16, dimensions[1] as u16, queue.clone()),
            _need_to_update_sc : true,
            _frame_finish_event : Some(sync::now(device.clone()).boxed()),
            _draw_list : Vec::new(),
            _camera : None,
            _sc_textures : Vec::new(),
            _postprocessor : Postprocessor::new(queue.clone(), dimensions[0] as u16, dimensions[1] as u16),
            _timer : Default::default()
        };
        result
    }

    fn resize(&mut self, width: u16, height: u16)
    {
        self._gbuffer = GBuffer::new(width, height, self._queue.clone());
        /* Создание узлов и связей графа постобработки */
        /* На данный момент это размытие в движении */
        self._postprocessor.reset();
        //let rh = self._postprocessor.rolling_hills(width, height, self._sc_textures[0].take().pix_fmt().clone()).unwrap();
        //let acc = self._postprocessor.acc_mblur_new(width, height, self._sc_textures[0].take().pix_fmt().clone()).unwrap();  // Создание ноды размытия в движении
        let acc = self._postprocessor.copy_node(width, height, self._sc_textures[0].take().pix_fmt().clone()).unwrap();  // Создание ноды размытия в движении
        self._postprocessor.link_stages(acc, 0, 0, format!("swapchain_out"));    // Соединение ноды с выходом.
        let mut camera = self._camera.as_ref().unwrap().lock().unwrap();
        camera.camera_mut().unwrap().set_projection(width as f32 / height as f32, 60.0 * 3.1415926535 / 180.0, 0.1, 100.0);
        
    }

    pub fn width(&self) -> u16
    {
        self._vk_surface.window().inner_size().width as u16
    }

    pub fn height(&self) -> u16
    {
        self._vk_surface.window().inner_size().height as u16
    }

    pub fn update_timer(&mut self, timer: &UniformTime)
    {
        //self._timer = timer.clone();
        self._postprocessor.timer = timer.clone();
        //self._postprocessor.uniform_to_all(&format!("timer"), timer);
    }

    /// Обновление swapchain изображений
    /// Как правило необходимо при изменении размера окна
    pub fn update_swapchain(&mut self)
    {
        let dimensions: [u32; 2] = self._vk_surface.window().inner_size().into();
        let (new_swapchain, new_images) =
            match self._swapchain.recreate(SwapchainCreateInfo{
                    image_extent: dimensions,
                    ..self._swapchain.create_info()
                }) {
                Ok(r) => r,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };
        self._aspect = dimensions[0] as f32 / dimensions[1] as f32;
        let dims = TextureDimensions::Dim2d{
            width: dimensions[0] as u32,
            height: dimensions[1] as u32,
            array_layers: 0
        };
        let db = Texture::new_empty_mutex("depth", dims, TexturePixelFormat::Depth16u, self._device.clone()).unwrap();
        
        self._swapchain = new_swapchain;
        //let dimensions = new_images[0].dimensions().width_height();

        self._framebuffers.clear();
        self._sc_textures.clear();
        for image in new_images
        {
            let cb = Texture::from_vk_image_view(ImageView::new_default(image.clone()).unwrap(), self._device.clone()).unwrap();
            cb.take_mut().set_vertical_address(crate::texture::TextureRepeatMode::ClampToEdge);
            cb.take_mut().set_horizontal_address(crate::texture::TextureRepeatMode::ClampToEdge);
            cb.take_mut().update_sampler();
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
    pub fn draw(&mut self, obj: RcBox<dyn GameObject>)
    {
        let owner = obj.lock().unwrap();
        let is_visual = owner.visual().is_some();
        if is_visual {
            self._draw_list.push(obj.clone());
            for child in owner.children()
            {
                self.draw(child);
            }
        }
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
        let gbuffer_fb = &mut *self._gbuffer._frame_buffer.take_mut();
        command_buffer_builder.bind_framebuffer(gbuffer_fb, gbuffer_rp.clone()).unwrap();
        //command_buffer_builder.set_cull_mode(vulkano::pipeline::graphics::rasterization::CullMode::Front);
        
        let camera_data = self._camera.as_ref().unwrap().lock().unwrap().camera().unwrap().uniform_data();
        for _obj in &self._draw_list
        {
            let locked = _obj.lock().unwrap();
            let obj = locked.visual().unwrap();
            obj.draw(&mut command_buffer_builder, &camera_data, gbuffer_rp.clone(), 0).unwrap();
        }

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
            return;
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
        
        // Передача входов в постобработку
        self._postprocessor.image_to_all(&format!("image"), &self._gbuffer._albedo);
        //self._postprocessor.image_to_all(&format!("vectors"), &self._gbuffer._vectors);

        // Подключение swapchain-изображения в качестве выхода
        self._postprocessor.set_output(format!("swapchain_out"), self._sc_textures[image_num].clone());
        
        // Построение прохода геометрии
        let gp_command_buffer = self.build_geametry_pass();
        // Построение прохода постобработки
        let pp_command_buffer = self._postprocessor.execute_graph();

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

    pub fn set_camera(&mut self, camera: RcBox<dyn GameObject>)
    {
        if camera.lock().unwrap().camera().is_some() {
            self._camera = Some(camera.clone());
        }
    }
}