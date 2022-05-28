pub mod postprocessor;
use vulkano::{device::{
    Device, DeviceExtensions, Features, Queue, DeviceCreateInfo, QueueCreateInfo,
    physical::{
        PhysicalDevice,
        PhysicalDeviceType
    }
}, swapchain::ColorSpace, render_pass::Subpass};
use vulkano::swapchain::{self, AcquireError, Swapchain, Surface, PresentMode, SwapchainCreateInfo};
use vulkano::image::{view::ImageView, SwapchainImage, ImageUsage, ImageLayout, SampleCount};
use vulkano::render_pass::{RenderPass, RenderPassCreateInfo, SubpassDescription, AttachmentDescription, AttachmentReference};
use vulkano::sync::{self, FlushError, GpuFuture};
use vulkano::instance::Instance;
#[allow(unused_imports)]
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, SecondaryAutoCommandBuffer};
use winit::window::{Window};
use crate::vulkano::device::DeviceOwned;
use std::sync::Arc;

use crate::texture::{Texture, TexturePixelFormat, TextureDimensions, TextureRef, TexturePixelFormatFeatures};
use crate::framebuffer::{Framebuffer, FramebufferRef, FramebufferBinder};
use crate::mesh::{MeshRef, Mesh};
use crate::shader::ShaderStructUniform;
use crate::references::*;
use crate::game_object::*;
use crate::components::*;
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
        //println!("Создание нового G-буфера {}x{}", width, height);
        let device = queue.device();
        let formats = [
            TexturePixelFormat::R8G8B8A8_UNORM,
            TexturePixelFormat::R8G8B8A8_UNORM,
            TexturePixelFormat::R8G8B8A8_UNORM,
            TexturePixelFormat::R16G16B16A16_SFLOAT,
            TexturePixelFormat::D16_UNORM
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
            let img_layout = match fmt.is_depth() {
                true => ImageLayout::DepthStencilAttachmentOptimal,
                false => ImageLayout::ColorAttachmentOptimal
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

//#[derive(Clone)]
pub enum RenderSurface
{
    Winit {
        surface: Arc<Surface<Window>>,
        swapchain: Arc<Swapchain<Window>>,
        swapchain_images: Vec<Arc<SwapchainImage<Window>>>,
        swapchain_textures: Vec<TextureRef>,
    },
    Offscreen {
        image: TextureRef,
        dimensions: [u16; 2]
    }
}

impl RenderSurface
{
    pub fn dimensions(&self) -> [u16; 2]
    {
        match self {
            Self::Winit{surface, ..} => surface.window().inner_size().into(),
            Self::Offscreen{image: _, dimensions} => *dimensions
        }
    }

    pub fn acquire_image(&self) -> Result<(TextureRef, usize, bool, Option<vulkano::swapchain::SwapchainAcquireFuture<Window>>), AcquireError>
    {
        match self {
            Self::Offscreen {image, ..} => Ok((image.clone(), 0, false, None)),
            Self::Winit{surface: _, swapchain, swapchain_images: _, swapchain_textures, ..} => {
                let (image_num, suboptimal, acquire_future) =
                match swapchain::acquire_next_image(swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => {
                        return Err(AcquireError::OutOfDate);
                    }
                    Err(e) => panic!("Failed to acquire next image: {:?}", e),
                };
                Ok((swapchain_textures[image_num].clone(), image_num, suboptimal, Some(acquire_future)))
            }
        }
    }

    pub fn winit(surface: Arc<Surface<Window>>, device: Arc<Device>, vsync: bool) -> Result<Self, String>
    {
        let physical_device = device.physical_device();
        let caps = physical_device.surface_capabilities(&surface, Default::default()).unwrap();
        let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();

        let formats: Vec<(vulkano::format::Format, ColorSpace)> =
            physical_device.surface_formats(&surface, Default::default()).unwrap()
            .into_iter()
            .filter_map(|a| {
                println!("{:?}: {:?}", a.0, a.1);
                match a {
                    (
                        vulkano::format::Format::R8G8B8_SRGB   |
                        vulkano::format::Format::R8G8B8A8_SRGB |
                        vulkano::format::Format::B8G8R8_SRGB   |
                        vulkano::format::Format::B8G8R8A8_SRGB |
                        vulkano::format::Format::A8B8G8R8_SRGB_PACK32,
                        ColorSpace::SrgbNonLinear
                    ) => Some(a),
                    _ => None
                }
            })
            .collect::<_>();
        
        let (swapchain, sc_images) = Swapchain::new(device.clone(), surface.clone(), SwapchainCreateInfo{
            min_image_count: caps.min_image_count,
            image_format: Some(formats[0].0),
            image_extent: surface.window().inner_size().into(),
            image_usage: ImageUsage::color_attachment(),
            composite_alpha: composite_alpha,
            present_mode: if vsync { PresentMode::Fifo } else { PresentMode::Immediate },
            ..Default::default()
        }).unwrap();
        Ok(RenderSurface::Winit {
            surface: surface,
            swapchain: swapchain,
            swapchain_images: sc_images.clone(),
            swapchain_textures: sc_images.iter().map(|image|{
                let cb = Texture::from_vk_image_view(
                    ImageView::new_default(image.clone()).unwrap(),
                    Some(image.clone()),
                    device.clone()
                ).unwrap();
                cb.take_mut().set_vertical_address(crate::texture::TextureRepeatMode::ClampToEdge);
                cb.take_mut().set_horizontal_address(crate::texture::TextureRepeatMode::ClampToEdge);
                cb.take_mut().update_sampler();
                cb
            }).collect::<Vec<_>>()
        })
    }

    pub fn update_swapchain(&mut self)
    {
        let (surface, swapchain) = match self {
            RenderSurface::Winit { ref surface, ref swapchain, .. } =>
                (surface.clone(), swapchain.clone()),
            RenderSurface::Offscreen { .. } => return
        };
        let sc_create_info = swapchain.create_info();
        let mut usage = sc_create_info.image_usage;
        usage.transfer_source = true;
        usage.transfer_destination = true;
        let dimensions: [u32; 2] = surface.window().inner_size().into();
        //surface
        //swapchain.device().physical_device().surface_capabilities(surface.as_ref(), surface_info);
        let (new_swapchain, new_images) =
            match swapchain.recreate(SwapchainCreateInfo{
                    image_extent: dimensions,
                    image_usage: usage,
                    ..sc_create_info
                }) {
                Ok(r) => r,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };
            
        let device = swapchain.device().clone();

        let mut sc_textures = Vec::new();
        //self._sc_textures.clear();
        for image in &new_images
        {
            let cb = Texture::from_vk_image_view(
                ImageView::new_default(image.clone()).unwrap(),
                Some(image.clone()),
                device.clone()
            ).unwrap();
            cb.take_mut().set_vertical_address(crate::texture::TextureRepeatMode::ClampToEdge);
            cb.take_mut().set_horizontal_address(crate::texture::TextureRepeatMode::ClampToEdge);
            cb.take_mut().update_sampler();
            sc_textures.push(cb);
        }
        match self {
            RenderSurface::Winit {
                surface: ref mut surf,
                ref mut swapchain,
                ref mut swapchain_images,
                ref mut swapchain_textures,.. } =>
                {
                    *surf = surface;
                    *swapchain = new_swapchain;
                    *swapchain_images = new_images;
                    *swapchain_textures = sc_textures;
                },
            RenderSurface::Offscreen { .. } => panic!()
        };
    }

    pub fn offscreen(device: Arc<Device>, dimensions: [u16; 2], pix_fmt: TexturePixelFormat) -> Result<Self, String>
    {
        Ok(Self::Offscreen {
            image: crate::texture::Texture::new_empty_mutex(
                "Offscreen surface",
                TextureDimensions::Dim2d {
                    width: dimensions[0] as _,
                    height: dimensions[1] as _,
                    array_layers: 1 
                },
                pix_fmt,
                device
            )?,
            dimensions: dimensions
        })
    }
}

/// Основная структура для рендеринга
pub struct Renderer
{
    _context : Arc<Instance>,
    _vk_surface : RenderSurface,
    _device : Arc<Device>,
    _queue : Arc<Queue>,
    //_swapchain : Option<(Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>)>,
    //_sc_textures : Vec<TextureRef>,

    _frame_finish_event : Option<Box<dyn GpuFuture + 'static>>,
    _need_to_update_sc : bool,

    _draw_list : Vec<RcBox<GameObject>>,
    
    _framebuffers : Vec<FramebufferRef>,
    _screen_plane : MeshRef,
    //_screen_plane_shader : ShaderProgramRef,
    _gbuffer : GBuffer,
    _aspect : f32,
    _camera : Option<RcBox<GameObject>>,
    _camera_data : CameraUniformData,

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

    pub fn render_result(&self) -> TextureRef
    {
        match self._vk_surface {
            RenderSurface::Offscreen { ref image, .. } =>
                image.clone(),
            RenderSurface::Winit { surface: _, swapchain: _, swapchain_images: _, ref swapchain_textures } =>
                swapchain_textures[0].clone()
        }
    }

    pub fn geometry_subpass(&self) -> Subpass
    {
        self._gbuffer._geometry_pass.clone().first_subpass()
    }

    pub fn camera_data(&self) -> CameraUniformData
    {
        self._camera_data
    }
}

use vulkano::device::DeviceCreationError;
#[allow(dead_code)]
impl Renderer
{
    fn default_device(vk_instance : Arc<Instance>) -> Result<(Arc<Device>, impl ExactSizeIterator<Item = Arc<Queue>>), DeviceCreationError>
    {
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
                        q.supports_graphics()
                        
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
        Device::new(physical_device, dev_info)
    }

    pub fn offscreen(vk_instance : Arc<Instance>, dimensions: [u16; 2]) -> Self
    {
        let (device, mut queues) = Self::default_device(vk_instance.clone()).unwrap();
        //let physical_device = device.physical_device();
        /*println!(
            "Используется устройство: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );*/

        let queue = queues.next().unwrap();
        let surface = RenderSurface::offscreen(device.clone(), dimensions, TexturePixelFormat::B8G8R8A8_SRGB).unwrap();
        let result = Renderer {
            _vk_surface : surface,
            _aspect : dimensions[0] as f32 / dimensions[1] as f32,
            _context : vk_instance,
            _device : device.clone(),
            _queue : queue.clone(),
            _framebuffers : Vec::new(),
            _screen_plane : Mesh::make_screen_plane(queue.clone()).unwrap(),
            _gbuffer : GBuffer::new(dimensions[0], dimensions[1], queue.clone()),
            _need_to_update_sc : true,
            _frame_finish_event : Some(sync::now(device.clone()).boxed()),
            _draw_list : Vec::new(),
            _camera : None,
            _camera_data : CameraUniformData::default(),
            _postprocessor : Postprocessor::new(queue.clone(), dimensions[0], dimensions[1]),
            _timer : Default::default()
        };
        result
    }

    pub fn winit(vk_instance : Arc<Instance>, surface: Arc<Surface<Window>>, vsync: bool) -> Self
    {
        let (device, mut queues) = Self::default_device(vk_instance.clone()).unwrap();
        //let physical_device = device.physical_device();
        /*println!(
            "Используется устройство: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );*/

        let queue = queues.next().unwrap();

        let surface = RenderSurface::winit(surface, device.clone(), vsync).unwrap();
        let dimensions = surface.dimensions();
        let result = Renderer {
            _vk_surface : surface,
            _aspect : dimensions[0] as f32 / dimensions[1] as f32,
            _context : vk_instance,
            _device : device.clone(),
            _queue : queue.clone(),
            _framebuffers : Vec::new(),
            _screen_plane : Mesh::make_screen_plane(queue.clone()).unwrap(),
            _gbuffer : GBuffer::new(dimensions[0], dimensions[1], queue.clone()),
            _need_to_update_sc : true,
            _frame_finish_event : Some(sync::now(device.clone()).boxed()),
            _draw_list : Vec::new(),
            _camera : None,
            _camera_data : CameraUniformData::default(),
            _postprocessor : Postprocessor::new(queue.clone(), dimensions[0], dimensions[1]),
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
        
        let acc = self._postprocessor.copy_node(width, height, TexturePixelFormat::B8G8R8A8_SRGB).unwrap();  // Создание ноды размытия в движении
        self._postprocessor.link_stages(acc, 0, 0, "swapchain_out".to_string());    // Соединение ноды с выходом.
        
        //let ((_istage, _input), (ostage, output)) = self._postprocessor.fidelityfx_super_resolution(width, height);
        //self._postprocessor.link_stages(ostage, output, 0, "swapchain_out".to_string());    // Соединение ноды с выходом.
        let self_camera = self._camera.clone();
        match self_camera {
            Some(ref camera) => {
                let mut _camera = camera.take();
                let cam_component_mutex = _camera.camera().unwrap().clone();
                let mut cam_component = cam_component_mutex.take_mut();
                cam_component.set_projection(width as f32 / height as f32, 60.0 * 3.1415926535 / 180.0, 0.1, 100.0);
                cam_component.on_render_init(&mut *_camera, self).unwrap();
            },
            None => ()
        }
        
    }

    pub fn width(&self) -> u16
    {
        self._vk_surface.dimensions()[0]
    }

    pub fn height(&self) -> u16
    {
        self._vk_surface.dimensions()[1]
    }

    pub fn update_timer(&mut self, timer: UniformTime)
    {
        //self._timer = timer.clone();
        self._postprocessor.timer = timer;
        //self._postprocessor.uniform_to_all(&format!("timer"), timer);
    }

    pub fn update_camera_data(&mut self, camera: CameraUniformData)
    {
        self._camera_data = camera.clone();
    }

    /// Обновление swapchain изображений
    /// Как правило необходимо при изменении размера окна
    pub fn update_swapchain(&mut self)
    {
        self._vk_surface.update_swapchain();
        let dimensions = self._vk_surface.dimensions();
        self.resize(dimensions[0], dimensions[1]);
    }

    /// Начинает проход геометрии
    pub fn begin_geametry_pass(&mut self)
    {
        self._draw_list.clear();
    }

    /// Передаёт объект для растеризации
    pub fn draw(&mut self, obj: RcBox<GameObject>)
    {
        let owner = obj.take();
        let is_visual = owner.visual().is_some();
        if is_visual {
            self._draw_list.push(obj.clone());
            for child in owner.children()
            {
                self.draw(child);
            }
        }
    }

    /*fn draw_thread(
        queue: Arc<Queue>,
        draw_list: Vec<RcBox<GameObject>>,
        subpass: vulkano::render_pass::Subpass,
        viewport: Viewport,
        thr_num: usize,
        thr_cnt: usize,
        camera_data: CameraUniformData
    ) -> SecondaryAutoCommandBuffer
    {
        let device = queue.device().clone();
        let mut last_meshmat_pair = (-1, -1);
        let queue_family = queue.family();
        let mut secondary = AutoCommandBufferBuilder::secondary_graphics(
            device.clone(),
            queue_family,
            CommandBufferUsage::OneTimeSubmit,
            subpass.clone()
        ).unwrap();
        secondary.set_viewport(0, [viewport.clone()]);
        let begin = draw_list.len()*thr_num/thr_cnt;
        let end = (draw_list.len()*(thr_num+1)/thr_cnt).min(draw_list.len());
        
        for _obj in &draw_list[begin..end]
        {
            let locked = _obj.take();
            let obj = locked.visual().unwrap();
            last_meshmat_pair = obj.draw(
                &*locked,
                &mut secondary,
                camera_data,
                subpass.clone(),
                last_meshmat_pair
            ).unwrap();
        }
        secondary.build().unwrap()
    }*/

    /// Фомирует буфер команд GPU
    pub fn build_geametry_pass(&mut self) -> PrimaryAutoCommandBuffer
    {
        //let timer = std::time::SystemTime::now();
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self._device.clone(),
            self._queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();
        let gbuffer_rp = self._gbuffer._geometry_pass.clone();
        {
            let gbuffer_fb = &mut *self._gbuffer._frame_buffer.take_mut();
            command_buffer_builder.bind_framebuffer(
                gbuffer_fb,
                gbuffer_rp.clone(),
                false
            ).unwrap();
        }

        match self._camera.clone() {
            Some(ref camera_object) => {
                let camera_locked = camera_object.take_mut();
                camera_locked.camera().unwrap().take_mut().on_render_init(&*camera_locked, self).unwrap();
            }
            None => ()
        }

        let mut last_meshmat_pair = (-1, -1);
        for _obj in self._draw_list.clone()
        {
            let locked = _obj.take();
            let mut visual = locked.visual().unwrap().take_mut();
            last_meshmat_pair = visual.on_geometry_pass(
                &*locked,
                self,
                &mut command_buffer_builder,
                last_meshmat_pair
            ).unwrap();
        }
        
        //let quarter = self._draw_list.len()/4;
        //let half = self._draw_list.len()/2;
        //let tquart = quarter * 3;
        /*let thr_cnt = 2;
        let mut tasks = Vec::new();
        for thr_num in 0..thr_cnt {
            let draw_list = self._draw_list.clone();
            let viewport = gbuffer_fb.viewport().clone();
            let subpass = gbuffer_rp.clone().first_subpass();
            let queue = self._queue.clone();
            let secondary = std::thread::spawn( move || {
                Self::draw_thread(
                    queue,
                    draw_list,
                    subpass,
                    viewport,
                    thr_num,thr_cnt,
                    camera_data)
            });
            tasks.push(secondary);
        }
        
        for task in tasks {
            let secondary_cb = task.join().unwrap();
            command_buffer_builder.execute_commands(secondary_cb).unwrap();
        }*/
        
        command_buffer_builder
            .end_render_pass().unwrap();
        let result = command_buffer_builder.build().unwrap();
        result
    }

    pub fn wait(&mut self)
    {
        //let future = self._frame_finish_event.as_mut().unwrap().clone();
        //self._frame_finish_event = Some(future.then_signal_semaphore_and_flush().unwrap());
        self._frame_finish_event.as_mut().unwrap().flush().unwrap();
        self._frame_finish_event.as_mut().unwrap().cleanup_finished();
        self._frame_finish_event = Some(sync::now(self.device().clone()).boxed());
    }

    /// Выполняет все сформированные буферы команд
    pub fn execute(&mut self, inputs: std::collections::HashMap<String, TextureRef>)
    {
        self._frame_finish_event.as_mut().unwrap().cleanup_finished();

        if self._need_to_update_sc {
            self.update_swapchain();
            self._need_to_update_sc = false;
        }

        let (sc_target, image_num, suboptimal, acquire_future) = self._vk_surface.acquire_image().unwrap();
        if suboptimal {
            self._need_to_update_sc = true;
            return;
        }
        // Построение прохода геометрии
        let gp_command_buffer = self.build_geametry_pass();
        //let geom_pass_time = timer.elapsed().unwrap().as_secs_f64();

        // Построение прохода постобработки
        // Передача входов в постобработку
        self._postprocessor.image_to_all(&"albedo".to_string(),   &self._gbuffer._albedo);
        //self._postprocessor.image_to_all(&format!("normals"),   &self._gbuffer._normals);
        //self._postprocessor.image_to_all(&format!("specromet"),   &self._gbuffer._specromet);
        //self._postprocessor.image_to_all(&format!("vectors"),   &self._gbuffer._vectors);
        //self._postprocessor.image_to_all(&format!("depth"),   &self._gbuffer._depth);
        if inputs.len() > 0 {
            for (name, img) in &inputs {
                self._postprocessor.image_to_all(name, img);
            }
        }
        //self._postprocessor.image_to_all(&format!("vectors"), &self._gbuffer._vectors);

        // Подключение swapchain-изображения в качестве выхода
        match self._vk_surface {
            RenderSurface::Winit { .. } => (),
            RenderSurface::Offscreen { .. } => self._postprocessor.set_output(format!("swapchain_out"), sc_target.clone())
        }
        let mut pp_command_buffer = self._postprocessor.execute_graph();
        /*let mut pp_command_buffer = AutoCommandBufferBuilder::primary(
            self._device.clone(),
            self._queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();*/
        let sc_out = self._postprocessor.get_output("swapchain_out".to_string()).unwrap();
        
        match self._vk_surface {
            RenderSurface::Winit { .. } => Texture::copy(&*(sc_out.take()), &*(sc_target.take()), Some(&mut pp_command_buffer), None),
            RenderSurface::Offscreen { .. } => ()
        };
        let pp_command_buffer = pp_command_buffer.build().unwrap();
        
        
        match self._vk_surface {
            RenderSurface::Winit { ref swapchain, .. } => {
                let future = self._frame_finish_event
                    .take().unwrap()
                    .then_execute(self._queue.clone(), gp_command_buffer).unwrap()
                    .join(acquire_future.unwrap())
                    .then_execute_same_queue(pp_command_buffer).unwrap()
                    .then_swapchain_present(self._queue.clone(), swapchain.clone(), image_num as _)
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
            },
            RenderSurface::Offscreen { .. } => {
                let future = self._frame_finish_event
                    .take().unwrap()
                    .then_execute(self._queue.clone(), gp_command_buffer).unwrap()
                    .then_execute_same_queue(pp_command_buffer).unwrap()
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

    pub fn set_camera(&mut self, camera: RcBox<GameObject>)
    {
        if camera.take().camera().is_some() {
            self._camera = Some(camera.clone());
        } else {
            panic!("Указанный объект не содержит компонента камеры");
        }
    }
}