mod postprocessor;
mod geometry_pass;
mod shadowmap_pass;
use vulkano::{device::{
    Device, DeviceExtensions, Features, Queue, DeviceCreateInfo, QueueCreateInfo,
    physical::{
        PhysicalDevice,
        PhysicalDeviceType
    }
}, swapchain::ColorSpace};
use vulkano::swapchain::{self, AcquireError, Swapchain, Surface, PresentMode, SwapchainCreateInfo};
use vulkano::image::{view::ImageView, SwapchainImage, ImageUsage};
use vulkano::sync::{self, FlushError, GpuFuture};
use vulkano::instance::Instance;
#[allow(unused_imports)]
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, SecondaryAutoCommandBuffer};
use winit::window::Window;
use crate::vulkano::device::DeviceOwned;
use std::sync::Arc;

use crate::texture::{Texture, TexturePixelFormat, TextureDimensions};
use crate::framebuffer::Framebuffer;
use crate::mesh::{MeshRef, Mesh};
use crate::references::*;
use crate::game_object::*;
use crate::components::*;
use crate::time::UniformTime;

pub use shadowmap_pass::ShadowMapPass;
pub use geometry_pass::GeometryPass;
pub use postprocessor::PostprocessingPass;

//#[derive(Clone)]
pub enum RenderSurface
{
    Winit {
        surface: Arc<Surface<Window>>,
        swapchain: Arc<Swapchain<Window>>,
        swapchain_images: Vec<Arc<SwapchainImage<Window>>>,
        swapchain_textures: Vec<Texture>,
    },
    Offscreen {
        image: Texture,
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

    pub fn acquire_image(&self) -> Result<(Texture, usize, bool, Option<vulkano::swapchain::SwapchainAcquireFuture<Window>>), AcquireError>
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
                let mut cb = Texture::from_vk_image_view(
                    ImageView::new_default(image.clone()).unwrap(),
                    device.clone()
                ).unwrap();
                cb.set_vertical_address(crate::texture::TextureRepeatMode::ClampToEdge);
                cb.set_horizontal_address(crate::texture::TextureRepeatMode::ClampToEdge);
                cb.update_sampler();
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
            let mut cb = Texture::from_vk_image_view(
                ImageView::new_default(image.clone()).unwrap(),
                device.clone()
            ).unwrap();
            cb.set_vertical_address(crate::texture::TextureRepeatMode::ClampToEdge);
            cb.set_horizontal_address(crate::texture::TextureRepeatMode::ClampToEdge);
            cb.update_sampler();
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
            image: crate::texture::Texture::new_empty(
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

//trait Cmpo : std::any::Any + Component {}
/// Основная структура для рендеринга
pub struct Renderer
{
    _context : Arc<Instance>,
    _vk_surface : RenderSurface,
    _device : Arc<Device>,
    _queue : Arc<Queue>,

    _frame_finish_event : Option<Box<dyn GpuFuture + 'static>>,
    _need_to_update_sc : bool,

    _draw_list   : Vec<(GOTransformUniform, RcBox<dyn Component>)>,
    _lights_list : Vec<(Light, Vec<(Framebuffer, ProjectionUniformData)>)>,
    
    _screen_plane : MeshRef,
    _aspect : f32,
    _camera : Option<RcBox<GameObject>>,
    _camera_data : ProjectionUniformData,
    
    _geometry_pass : GeometryPass,
    _postprocessor : PostprocessingPass,
    _shadowmap_pass: ShadowMapPass,
    _screen_font : Texture,
    
    _timer : UniformTime
}

impl Renderer
{
    pub fn postprocessor(&mut self) -> &mut PostprocessingPass
    {
        &mut self._postprocessor
    }

    pub fn render_result(&self) -> &Texture
    {
        match self._vk_surface {
            RenderSurface::Offscreen { ref image, .. } =>
                image,
            RenderSurface::Winit { surface: _, swapchain: _, swapchain_images: _, ref swapchain_textures } =>
                &swapchain_textures[0]
        }
    }

    /*pub fn geometry_pass(&self) -> &GeometryPass
    {
        &self._geometry_pass
    }

    pub fn camera_data(&self) -> ProjectionUniformData
    {
        self._camera_data
    }*/

    pub fn add_renderable_component<T: Component + Clone>(&mut self, transform_data: GOTransformUniform, component: &T)
    {
        self._draw_list.push((transform_data, RcBox::construct(component.clone())))
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
        
        println!("Используется {}", physical_device.properties().device_name);
        Device::new(physical_device, dev_info)
    }

    pub fn offscreen(vk_instance : Arc<Instance>, dimensions: [u16; 2]) -> Self
    {
        let (device, mut queues) = Self::default_device(vk_instance.clone()).unwrap();

        let queue = queues.next().unwrap();
        let surface = RenderSurface::offscreen(device.clone(), dimensions, TexturePixelFormat::B8G8R8A8_SRGB).unwrap();
        let result = Renderer {
            _vk_surface : surface,
            _aspect : dimensions[0] as f32 / dimensions[1] as f32,
            _context : vk_instance,
            _device : device.clone(),
            _queue : queue.clone(),
            _screen_plane : Mesh::make_screen_plane(queue.clone()).unwrap(),
            _geometry_pass : GeometryPass::new(dimensions[0], dimensions[1], queue.clone()),
            _need_to_update_sc : true,
            _frame_finish_event : Some(sync::now(device.clone()).boxed()),
            _draw_list : Vec::new(),
            _lights_list : Vec::new(),
            _camera : None,
            _camera_data : ProjectionUniformData::default(),
            _postprocessor : PostprocessingPass::new(queue.clone(), dimensions[0], dimensions[1]),
            _shadowmap_pass : ShadowMapPass::new(queue.clone()),
            _screen_font : Texture::from_file(queue.clone(), "data/texture/shadertoy_font.png").unwrap(),
            _timer : Default::default()
        };
        result
    }

    pub fn winit(vk_instance : Arc<Instance>, surface: Arc<Surface<Window>>, vsync: bool) -> Self
    {
        let (device, mut queues) = Self::default_device(vk_instance.clone()).unwrap();

        let queue = queues.next().unwrap();

        let surface = RenderSurface::winit(surface, device.clone(), vsync).unwrap();
        let dimensions = surface.dimensions();
        let result = Renderer {
            _vk_surface : surface,
            _aspect : dimensions[0] as f32 / dimensions[1] as f32,
            _context : vk_instance,
            _device : device.clone(),
            _queue : queue.clone(),
            _screen_plane : Mesh::make_screen_plane(queue.clone()).unwrap(),
            _need_to_update_sc : true,
            _frame_finish_event : Some(sync::now(device.clone()).boxed()),
            _draw_list : Vec::new(),
            _lights_list : Vec::new(),
            _camera : None,
            _camera_data : ProjectionUniformData::default(),
            _shadowmap_pass : ShadowMapPass::new(queue.clone()),
            _geometry_pass : GeometryPass::new(dimensions[0], dimensions[1], queue.clone()),
            _postprocessor : PostprocessingPass::new(queue.clone(), dimensions[0], dimensions[1]),
            _screen_font : Texture::from_file(queue.clone(), "data/texture/shadertoy_font.png").unwrap(),
            _timer : Default::default()
        };
        result
    }

    fn resize(&mut self, width: u16, height: u16)
    {
        self._geometry_pass = GeometryPass::new(width, height, self._queue.clone());
        /* Создание узлов и связей графа постобработки */
        /* На данный момент это размытие в движении */
        self._postprocessor.reset();
        
        //let rh = self._postprocessor.rolling_hills(width, height, self._sc_textures[0].take().pix_fmt().clone()).unwrap();
        //let acc = self._postprocessor.acc_mblur_new(width, height, self._sc_textures[0].take().pix_fmt().clone()).unwrap();  // Создание ноды размытия в движении
        
        let acc = self._postprocessor.copy_node(width, height, TexturePixelFormat::R8G8B8A8_UNORM).unwrap();  // Создание ноды размытия в движении
        self._postprocessor.link_stages(acc, 0, 0, "swapchain_out".to_string());    // Соединение ноды с выходом.
        
        //let ((_istage, _input), (ostage, output)) = self._postprocessor.fidelityfx_super_resolution(width, height);
        //self._postprocessor.link_stages(ostage, output, 0, "swapchain_out".to_string());    // Соединение ноды с выходом.
        let self_camera = self._camera.clone();
        match self_camera {
            Some(ref camera) => {
                let mut _camera = camera.take();
                _camera.camera_mut().unwrap().set_aspect_dimenstions(width, height);
                let cam_component = _camera.camera().unwrap();
                self._camera_data = cam_component.uniform_data(&*_camera);
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

    pub fn update_camera_data(&mut self, camera: ProjectionUniformData)
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
        self._lights_list.clear();
    }

    /// Передаёт объект для растеризации
    pub fn draw(&mut self, obj: RcBox<GameObject>)
    {
        let mut owner = obj.take();
        match owner.visual() {
            Some(visual) => {
                self.add_renderable_component(owner.transform().uniform_value(), visual);
            },
            None => ()
        }
        match owner.light() {
            Some(_light) => {
                let light = (_light.clone(), _light.framebuffers(&*owner));
                self._lights_list.push(light);
            },
            None => ()
        }
        for component in owner.get_all_components().clone()
        {
            let mut cmp = component.lock().unwrap();
            cmp.on_loop(&mut *owner);
        }
        for child in owner.children()
        {
            self.draw(child);
        }
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
    pub fn execute(&mut self, inputs: std::collections::HashMap<String, Texture>)
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

        // Проход карт теней
        let mut sm_command_buffers = Vec::new();
        for (_, frame_buffers) in &self._lights_list
        {
            for (shadow_buffer, projection_data) in frame_buffers {
                let command_buffer = self._shadowmap_pass.build_shadow_map_pass(
                    &mut shadow_buffer.clone(),
                    *projection_data,
                    self._draw_list.clone()
                );
                sm_command_buffers.push(command_buffer);
            }
        }

        // Построение прохода геометрии
        let gp_command_buffer = self._geometry_pass.build_geometry_pass(self._camera_data, self._draw_list.clone());
        //let geom_pass_time = timer.elapsed().unwrap().as_secs_f64();

        // Построение прохода постобработки
        // Передача входов в постобработку
        self._postprocessor.image_to_all(&"albedo".to_string(),   self._geometry_pass.albedo());
        self._postprocessor.image_to_all(&"font".to_string(), &self._screen_font);
        self._postprocessor.uniform_to_all(&"camera".to_string(), self._camera_data);
        self._postprocessor.uniform_to_all(&"light".to_string(), self._lights_list[0].1[0].1);
        let fb = &self._lights_list[0].0;
        match fb.shadowmap() {
            Some(sm_buffer) => self._postprocessor.image_to_all(&"shadowmap".to_string(), sm_buffer),
            None => ()
        };
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
        let sc_out = self._postprocessor.get_output("swapchain_out".to_string()).unwrap();
        
        match self._vk_surface {
            RenderSurface::Winit { .. } => Texture::copy(sc_out, &sc_target, Some(&mut pp_command_buffer), None),
            RenderSurface::Offscreen { .. } => ()
        };
        let pp_command_buffer = pp_command_buffer.build().unwrap();
        
        let mut future = self._frame_finish_event
            .take().unwrap()
            .then_execute(self._queue.clone(), gp_command_buffer).unwrap().boxed();
        for cb in sm_command_buffers {
            future = future.then_execute_same_queue(cb).unwrap().boxed();
        }
        match self._vk_surface {
            RenderSurface::Winit { .. } => {
                future = future.join(acquire_future.unwrap()).boxed();
            },
            _ => ()
        }
        future = future.then_execute_same_queue(pp_command_buffer).unwrap().boxed();
        match self._vk_surface {
            RenderSurface::Winit { ref swapchain, .. } => {
                future = future.then_swapchain_present(self._queue.clone(), swapchain.clone(), image_num as _).boxed();
            },
            _ => ()
        };
        let future = future.then_signal_fence_and_flush();
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

        /*match self._vk_surface {
            RenderSurface::Winit { ref swapchain, .. } => {
                let mut future = self._frame_finish_event
                    .take().unwrap()
                    .then_execute(self._queue.clone(), gp_command_buffer).unwrap().boxed();
                for cb in sm_command_buffers {
                    future = future.then_execute(self._queue.clone(), cb).unwrap().boxed();
                }
                let future = future.join(acquire_future.unwrap())
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
        };*/
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
