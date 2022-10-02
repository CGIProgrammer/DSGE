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
use crate::{vulkano::device::DeviceOwned, texture::TextureCommandSet, components::light::{LightsUniformData, SpotLightUniform}, types::{Vec3, Mat4}, mesh::MeshView};
use std::{sync::Arc, collections::HashMap, cmp::Ordering};

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

pub(crate) const LIGHT_DATA_MAX_SIZE: usize = 96;
pub(crate) const LIGHT_MAX_COUNT: usize = 64;

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
        usage.transfer_src = true;
        usage.transfer_dst = true;
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

/// Основная структура для рендеринга
pub struct Renderer
{
    _context : Arc<Instance>,
    _vk_surface : RenderSurface,
    _device : Arc<Device>,
    _queue : Arc<Queue>,

    _frame_finish_event : Option<Box<dyn GpuFuture + 'static>>,
    _need_to_update_sc : bool,

    _draw_list   : Vec<(GOTransformUniform, Arc<MeshVisual>)>,
    _lights_list : Vec<(Light, Vec<(Framebuffer, ProjectionUniformData)>)>,
    _lights_data : Texture,
    
    _screen_plane : MeshRef,
    _aspect : f32,
    _camera : Option<RcBox<GameObject>>,
    _camera_data : ProjectionUniformData,
    
    _geometry_pass : GeometryPass,
    _postprocessor : PostprocessingPass,
    _shadowmap_pass: ShadowMapPass,
    
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

    pub fn add_renderable_component(&mut self, transform_data: GOTransformUniform, component: Arc<MeshVisual>)
    {
        self._draw_list.push((transform_data, component))
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
            khr_buffer_device_address: true,
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
            enabled_extensions: device_extensions, /*physical_device
                .supported_extensions()
                .union(&device_extensions),*/
            ..Default::default()
        };
        
        println!("Используется {}", physical_device.properties().device_name);
        //println!("{dev_info:?}");
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
            _lights_data : Texture::new_empty(
                "light_data",
                TextureDimensions::Dim2d{width : (LIGHT_DATA_MAX_SIZE/4) as _, height : (LIGHT_MAX_COUNT*2) as _, array_layers : 1},
                TexturePixelFormat::R32G32B32A32_SFLOAT,
                device.clone()
            ).unwrap(),
            _camera : None,
            _camera_data : ProjectionUniformData::default(),
            _postprocessor : PostprocessingPass::new(queue.clone(), dimensions[0], dimensions[1]),
            _shadowmap_pass : ShadowMapPass::new(queue.clone()),
            _timer : Default::default()
        };
        result
    }

    pub fn winit(vk_instance : Arc<Instance>, surface: Arc<Surface<Window>>, vsync: bool) -> Self
    {
        let (device, mut queues) = Self::default_device(vk_instance.clone()).unwrap();

        let queue = queues.next().unwrap();
        for i in device.active_queue_families() {
            println!("queues_count {}, supports_graphics {}", i.queues_count(), i.supports_graphics());
        }
        
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
            _lights_data : Texture::new_empty(
                "light_data",
                TextureDimensions::Dim2d{width : (LIGHT_DATA_MAX_SIZE/4) as _, height : (LIGHT_MAX_COUNT*2) as _, array_layers : 1},
                TexturePixelFormat::R32G32B32A32_SFLOAT,
                device.clone()
            ).unwrap(),
            _camera : None,
            _camera_data : ProjectionUniformData::default(),
            _shadowmap_pass : ShadowMapPass::new(queue.clone()),
            _geometry_pass : GeometryPass::new(dimensions[0], dimensions[1], queue.clone()),
            _postprocessor : PostprocessingPass::new(queue.clone(), dimensions[0], dimensions[1]),
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
        
        let acc = self._postprocessor.copy_node(width, height, TexturePixelFormat::B8G8R8A8_SRGB).unwrap();  // Создание ноды размытия в движении
        self._postprocessor.link_stages(acc, 0, acc, "accumulator_in".to_owned());    // Соединение ноды с выходом.
        self._postprocessor.link_stages(acc, 1, 0, "swapchain_out".to_owned());    // Соединение ноды с выходом.
        
        //let ((_istage, _input), (ostage, output)) = self._postprocessor.fidelityfx_super_resolution(width, height);
        //self._postprocessor.link_stages(ostage, output, 0, "swapchain_out".to_owned());    // Соединение ноды с выходом.
        let self_camera = self._camera.clone();
        match self_camera {
            Some(ref camera) => {
                let mut _camera = camera.lock();
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

    /// Передаёт объект для прохода геометрии
    pub fn draw(&mut self, obj: RcBox<GameObject>)
    {
        let mut owner = obj.lock();
        let owner_transform = owner.transform.clone();
        match owner.visual() {
            Some(visual) => {
                self.add_renderable_component(owner.transform().uniform_value(), Arc::new(visual.clone()));
            },
            None => ()
        }
        match owner.light_mut() {
            Some(_light) => {
                let light = (_light.clone(), _light.framebuffers(&owner_transform));
                self._lights_list.push(light);
            },
            None => ()
        }
        for child in owner.children().clone()
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

    fn build_lights_buffer(&mut self) -> (PrimaryAutoCommandBuffer, LightsUniformData)
    {
        let mut light_data = [[[0.0f32; LIGHT_DATA_MAX_SIZE]; 2]; LIGHT_MAX_COUNT];
        let mut lights_count: usize = 0;
        let (mut spotlights, mut spotlight_shadowmaps_count) = (0i32, 0i32);
        let (mut point_lights, mut point_light_shadowmaps_count) = (0i32, 0i32);
        let (mut sun_lights, mut sun_light_shadowmaps_count) = (0i32, 0i32);

        self._lights_list.sort_by(|(light_a, _), (light_b, _)| light_a.cmp(light_b));

        // Сериализация источников света в текстуру
        for (light, frame_buffers) in &self._lights_list {
            let mut serialized = light.serialize();
            match light {
                Light::Spot(_) => {
                    spotlights += 1;
                    if let Some(_) = light.shadowmap() {
                        serialized.push(spotlight_shadowmaps_count as _);
                        spotlight_shadowmaps_count += 1;
                    } else {
                        serialized.push(-1.0);
                    }
                },
                Light::Point(_) => {
                    point_lights += 1;
                    if let Some(_) = light.shadowmap() {
                        serialized.push(point_light_shadowmaps_count as _);
                        point_light_shadowmaps_count += 1;
                    } else {
                        serialized.push(-1.0);
                    }
                },
                Light::Sun(_) => {
                    sun_lights += 1;
                    if let Some(_) = light.shadowmap() {
                        serialized.push(sun_light_shadowmaps_count as _);
                        sun_light_shadowmaps_count += 1;
                    } else {
                        serialized.push(-1.0);
                    }
                }
            }
            for (i, num) in serialized.iter().enumerate() {
                light_data[lights_count][0][i] = *num;
            }
            for (i, (_, projection_data)) in frame_buffers.iter().enumerate() {
                let matrix = projection_data.full_matrix();
                light_data[lights_count][1][i*16..i*16+16].copy_from_slice(matrix.as_slice());
            }
            lights_count += 1;
        }
        //let ld: [[[f32; 4]; LIGHT_DATA_MAX_SIZE / 4]; LIGHT_MAX_COUNT * 2] = unsafe { transmute_copy(&light_data) };
        //println!("{:?}", ld);
        //let (light, _) = &self._lights_list[0];
        //panic!("{:?}", light);

        let mut lccbb = AutoCommandBufferBuilder::primary(
            self._device.clone(),
            self._queue.family(),
            CommandBufferUsage::OneTimeSubmit
        ).unwrap();
        lccbb
            .update_data(&self._lights_data, light_data.to_vec()).unwrap();
        let light_compile_cb = lccbb.build().unwrap();
        (light_compile_cb, LightsUniformData::new(spotlights, point_lights, sun_lights))
    }

    /// Выполняет все сформированные буферы команд
    pub fn execute(&mut self, inputs: &HashMap<String, Texture>)
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

        /*let mut draw_list = HashMap::new();
        for (transform,visual) in &self._draw_list {
            let mat_id = visual.material_id();
            let mesh_id = visual.mesh().buffer_id();
            let material_list = match draw_list.get_mut(&mat_id) {
                Some(material_list) => material_list,
                None => {
                    let material_list = HashMap::<i32, Vec<(GOTransformUniform, Arc<MeshVisual>)>>::new();
                    draw_list.insert(mat_id, material_list);
                    draw_list.get_mut(&mat_id).unwrap()
                }
            };
            let mesh_buffer_list = match material_list.get_mut(&mesh_id) {
                Some(mesh_list) => {
                    mesh_list
                },
                None => {
                    let mesh_list = Vec::<(GOTransformUniform, Arc<MeshVisual>)>::new();
                    material_list.insert(mesh_id, mesh_list);
                    material_list.get_mut(&mesh_id).unwrap()
                }
            };
            mesh_buffer_list.push((transform, visual).clone());
        }*/
        
        // Проход карт теней
        let mut sm_command_buffers = Vec::new();
        for (_, frame_buffers) in &self._lights_list {
            for (shadow_buffer, projection_data) in frame_buffers {
                let command_buffer = self._shadowmap_pass.build_shadow_map_pass(
                    &mut shadow_buffer.clone(),
                    *projection_data,
                    &self._draw_list
                );
                sm_command_buffers.push(command_buffer);
            }
        }

        // Сборка буфера с информацией об источниках света
        let (light_compile_cb, light_count) = self.build_lights_buffer();

        let cam_obj = self._camera.as_ref().unwrap().lock();
        let component_camera = cam_obj.camera().unwrap().clone();
        self._camera_data = component_camera.uniform_data(&*cam_obj);

        // Построение прохода геометрии
        let gp_command_buffer = self._geometry_pass.build_geometry_pass(self._camera_data, self._draw_list.clone());

        // Построение прохода постобработки
        // Передача входов в постобработку
        self._postprocessor.image_to_all(&"gAlbedo".to_owned(),   self._geometry_pass.albedo());
        self._postprocessor.image_to_all(&"gNormals".to_owned(),  self._geometry_pass.normals());
        self._postprocessor.image_to_all(&"gMasks".to_owned(), self._geometry_pass.specromet());
        self._postprocessor.image_to_all(&"gDepth".to_owned(), self._geometry_pass.depth());
        self._postprocessor.image_to_all(&"lights_data".to_owned(), &self._lights_data);
        self._postprocessor.uniform_to_all(&"lights_count".to_owned(), light_count);
        self._postprocessor.uniform_to_all(&"camera".to_owned(), self._camera_data);

        for (li, fbs) in &self._lights_list {
            if let Light::Spot(light) = li {
                let lidata = light.as_uniform_struct(fbs[0].1.full_matrix(), 0);
                self._postprocessor.uniform_to_all(&"testing_light".to_owned(), lidata);
            }
        }
        //self._postprocessor
        let mut spot_shadowmaps = vec![];
        let mut sun_shadowmaps = vec![];
        let mut point_shadowmaps = vec![];
        if self._lights_list.len() > 0 {
            //let light = &self._lights_list[0].0;
            for (light, _) in &self._lights_list {
                if let Some(sm_buffer) =  light.shadowmap() {
                    match light {
                        Light::Spot(_) => spot_shadowmaps.push(sm_buffer),// self._postprocessor.image_array_to_all(&"spot_shadowmaps[4]".to_owned(), &[sm_buffer], 0),
                        Light::Point(_) => point_shadowmaps.push(sm_buffer),// self.__postprocessor.image_array_to_all(&"point_shadowmaps[4]".to_owned(), &[sm_buffer], 0),
                        Light::Sun(_) => sun_shadowmaps.push(sm_buffer), //self._postprocessor.image_array_to_all(&"sun_shadowmaps[4]".to_owned(), &[sm_buffer], 0),
                    };
                };
            }
        }
        //dbg!(spot_shadowmaps.clone());
        //dbg!(point_shadowmaps.clone());
        let sm = *spot_shadowmaps.last().unwrap();
        while spot_shadowmaps.len() < 4 {
            spot_shadowmaps.push(sm)
        }
        let sm = *point_shadowmaps.last().unwrap();
        while point_shadowmaps.len() < 4 {
            point_shadowmaps.push(sm)
        }
        self._postprocessor.image_array_to_all(&"spot_shadowmaps[4]".to_owned(), spot_shadowmaps.as_slice(), 0);
        self._postprocessor.image_array_to_all(&"point_shadowmaps[4]".to_owned(), point_shadowmaps.as_slice(), 0);
        /*self._postprocessor.image_to_all(&format!("normals"),   &self._geometry_pass.normals());
        self._postprocessor.image_to_all(&format!("specromet"),   &self._geometry_pass.specromet());
        self._postprocessor.image_to_all(&format!("vectors"),   &self._geometry_pass.vectors());
        self._postprocessor.image_to_all(&format!("depth"),   &self._geometry_pass.depth());*/
        if inputs.len() > 0 {
            for (name, img) in inputs {
                self._postprocessor.image_to_all(name, img);
            }
        }
        //self._postprocessor.image_to_all(&format!("vectors"), &self._gbuffer._vectors);

        // Подключение swapchain-изображения в качестве выхода
        match self._vk_surface {
            RenderSurface::Winit { .. } => (self._postprocessor.set_output(format!("swapchain_out"), sc_target.clone())),
            RenderSurface::Offscreen { .. } => self._postprocessor.set_output(format!("swapchain_out"), sc_target.clone())
        }
        let pp_command_buffer = self._postprocessor.execute_graph();
        let _sc_out = self._postprocessor.get_output("swapchain_out".to_owned()).unwrap();
        /*let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self._device.clone(),
            self._queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();*/

        match self._vk_surface {
            RenderSurface::Winit { .. } => (), //Texture::copy(_sc_out, &sc_target, Some(&mut pp_command_buffer), None),
            RenderSurface::Offscreen { .. } => ()
        };
        //let tex_copy_cb = command_buffer_builder.build().unwrap();
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
        future = future
            .then_execute_same_queue(light_compile_cb).unwrap()
            .then_execute_same_queue(pp_command_buffer).unwrap().boxed();
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
        if camera.lock().camera().is_some() {
            self._camera = Some(camera.clone());
        } else {
            panic!("Указанный объект не содержит компонента камеры");
        }
    }

    pub fn camera(&mut self) -> Option<GameObjectRef>
    {
        self._camera.clone()
    }
}
