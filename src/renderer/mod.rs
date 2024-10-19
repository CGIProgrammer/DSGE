mod geometry_pass;
mod postprocessor;
mod shadowmap_pass;
use crate::{
    command_buffer::CommandBufferFather,
    components::light::{LightShaderStruct, LightsUniformData, PointLightUniform, ShadowBuffer, ShadowMapMode, SpotlightUniform, SunLightUniform},
    resource_manager::{ResourceManager, ResourceManagerRef, MAX_POINT_LIGHTS, MAX_SPOTLIGHTS, MAX_SUN_LIGHTS},
    texture::{Texture, TextureCommandSet, TextureUseCase},
    types::{ArrayInto, Mat4, Vec4},
    vulkano::device::DeviceOwned,
};
use std::{cmp::Ordering, collections::HashMap, sync::Arc};
#[allow(unused_imports)]
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
    SecondaryAutoCommandBuffer,
};
use vulkano::{descriptor_set::allocator::{StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo}, device::QueueFlags, image::{view::ImageView, Image, ImageUsage}, memory::{allocator::{BuddyAllocator, BumpAllocator, GenericMemoryAllocator, GenericMemoryAllocatorCreateInfo, StandardMemoryAllocator}, MemoryProperties, MemoryPropertyFlags}, swapchain::{SurfaceInfo, SwapchainPresentInfo}, DeviceSize, Validated, VulkanError};
use vulkano::instance::Instance;
use vulkano::swapchain::{
    self, PresentMode, Surface, Swapchain, SwapchainCreateInfo,
};
use vulkano::sync::{self, GpuFuture};
use vulkano::{
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Features, Queue,
        QueueCreateInfo,
    },
    swapchain::ColorSpace,
};
use winit::window::Window;

use crate::components::*;
use crate::game_object::*;
use crate::mesh::{Mesh, MeshRef};
use crate::references::*;
use crate::texture::{TextureDimensions, TexturePixelFormat};
use crate::time::UniformTime;
use crate::texture::TextureViewType;

pub use geometry_pass::GeometryPass;
pub use postprocessor::{PostprocessingPass, RenderResolution};
pub use shadowmap_pass::ShadowMapPass;

pub(crate) type BumpMemoryAllocator = GenericMemoryAllocator<BumpAllocator>;

    /// Creates a new `StandardMemoryAllocator` with default configuration.
pub fn bump_memory_allocator_new_default(device: Arc<Device>) -> BumpMemoryAllocator {
    let MemoryProperties {
        memory_types,
        memory_heaps,
        ..
    } = device.physical_device().memory_properties();

    let mut block_sizes = vec![0; memory_types.len()];
    let mut memory_type_bits = u32::MAX;

    for (index, memory_type) in memory_types.iter().enumerate() {
        const LARGE_HEAP_THRESHOLD: DeviceSize = 1024 * 1024 * 1024;

        let heap_size = memory_heaps[memory_type.heap_index as usize].size;

        block_sizes[index] = if heap_size >= LARGE_HEAP_THRESHOLD {
            256 * 1024
        } else {
            64 * 1024
        };

        if memory_type.property_flags.intersects(
            MemoryPropertyFlags::LAZILY_ALLOCATED
                | MemoryPropertyFlags::PROTECTED
                | MemoryPropertyFlags::DEVICE_COHERENT
                | MemoryPropertyFlags::RDMA_CAPABLE,
        ) {
            // VUID-VkMemoryAllocateInfo-memoryTypeIndex-01872
            // VUID-vkAllocateMemory-deviceCoherentMemory-02790
            // Lazily allocated memory would just cause problems for suballocation in general.
            memory_type_bits &= !(1 << index);
        }
    }

    let create_info = GenericMemoryAllocatorCreateInfo {
        block_sizes: &block_sizes,
        memory_type_bits,
        ..Default::default()
    };

    BumpMemoryAllocator::new(device, create_info)
}

//pub(crate) const LIGHT_DATA_MAX_SIZE: usize = 96;
//pub(crate) const LIGHT_MAX_COUNT: usize = 64;

//#[derive(Clone)]
pub enum RenderSurface {
    Winit {
        surface: Arc<Surface>,
        swapchain: Arc<Swapchain>,
        swapchain_images: Vec<Arc<Image>>,
        swapchain_textures: Vec<Texture>,
    },
    Offscreen {
        image: Texture,
        dimensions: [u16; 2],
    },
}

impl RenderSurface {
    pub fn window(&self) -> Option<&Window>
    {
        if let Self::Winit { surface, ..} = self {
            surface.object().unwrap().downcast_ref::<Window>()
        } else {
            None
        }
    }

    pub fn dimensions(&self) -> [u16; 2] {
        match self {
            Self::Winit {  .. } => self.window().unwrap().inner_size().into(),
            Self::Offscreen {
                image: _,
                dimensions,
            } => *dimensions,
        }
    }

    pub fn acquire_image(
        &self,
    ) -> Result<
        (
            Texture,
            usize,
            bool,
            Option<vulkano::swapchain::SwapchainAcquireFuture>,
        ),
        String,
    > {
        match self {
            Self::Offscreen { image, .. } => Ok((image.clone(), 0, false, None)),
            Self::Winit {
                surface: _,
                swapchain,
                swapchain_images: _,
                swapchain_textures,
                ..
            } => {
                let (image_num, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };
                Ok((
                    swapchain_textures[image_num as usize].clone(),
                    image_num as _,
                    suboptimal,
                    Some(acquire_future),
                ))
            }
        }
    }

    pub fn winit(
        surface: Arc<Surface>,
        device: Arc<Device>,
        vsync: bool,
    ) -> Result<Self, String> {
        let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
        let physical_device = device.physical_device();
        let caps = physical_device
            .surface_capabilities(&surface, Default::default())
            .unwrap();
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();

        let formats: Vec<(vulkano::format::Format, ColorSpace)> = physical_device
            .surface_formats(&surface, Default::default())
            .unwrap()
            .into_iter()
            .filter_map(|a| {
                println!("{:?}: {:?}", a.0, a.1);
                match a {
                    (
                        vulkano::format::Format::R8G8B8_SRGB
                        | vulkano::format::Format::R8G8B8A8_SRGB
                        | vulkano::format::Format::B8G8R8_SRGB
                        | vulkano::format::Format::B8G8R8A8_SRGB
                        | vulkano::format::Format::A8B8G8R8_SRGB_PACK32,
                        ColorSpace::SrgbNonLinear,
                    ) => Some(a),
                    _ => None,
                }
            })
            .collect::<_>();
        
        
        let (swapchain, sc_images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count, //.max(2),
                image_format: formats[0].0,
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: composite_alpha,
                present_mode: if vsync {
                    PresentMode::Fifo
                } else {
                    PresentMode::Immediate
                },
                ..Default::default()
            },
        )
        .unwrap();
        Ok(RenderSurface::Winit {
            surface: surface,
            swapchain: swapchain,
            swapchain_images: sc_images.clone(),
            swapchain_textures: sc_images
                .iter()
                .map(|image| {
                    let mut cb = Texture::from_vk_image_view(
                        ImageView::new_default(image.clone()).unwrap(),
                        device.clone(),
                    )
                    .unwrap();
                    cb.set_address_mode([crate::texture::TextureRepeatMode::ClampToEdge; 3]);
                    cb
                })
                .collect::<Vec<_>>(),
        })
    }

    pub fn update_swapchain(&mut self) {
        let (window, surface, swapchain) = match self {
            RenderSurface::Winit {
                ref surface,
                ref swapchain,
                ..
            } => (self.window().unwrap(), surface.clone(), swapchain.clone()),
            RenderSurface::Offscreen { .. } => return,
        };
        let sc_create_info = swapchain.create_info();
        let mut usage = sc_create_info.image_usage;
        usage |= ImageUsage::TRANSFER_SRC;
        usage |= ImageUsage::TRANSFER_DST;
        let mut dimensions: [u32; 2] = window.inner_size().into();
        //surface
        //swapchain.device().physical_device().surface_capabilities(surface.as_ref(), surface_info);
        let surface_info = SurfaceInfo {
            full_screen_exclusive: sc_create_info.full_screen_exclusive,
            win32_monitor: sc_create_info.win32_monitor,
            ..Default::default()
        };
        let surf_caps = swapchain
            .device()
            .physical_device()
            .surface_capabilities(surface.as_ref(), surface_info)
            .unwrap();
        if !(surf_caps.min_image_extent[0]..=surf_caps.max_image_extent[0]).contains(&dimensions[0]) ||
           !(surf_caps.min_image_extent[1]..=surf_caps.max_image_extent[1]).contains(&dimensions[1]) {
            dimensions = surf_caps.min_image_extent;
        }
        let (new_swapchain, new_images) = match swapchain.recreate(SwapchainCreateInfo {
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
        for image in &new_images {
            let mut cb = Texture::from_vk_image_view(
                ImageView::new_default(image.clone()).unwrap(),
                device.clone(),
            )
            .unwrap();
            cb.set_address_mode([crate::texture::TextureRepeatMode::ClampToEdge; 3]);
            sc_textures.push(cb);
        }
        match self {
            RenderSurface::Winit {
                surface: ref mut surf,
                ref mut swapchain,
                ref mut swapchain_images,
                ref mut swapchain_textures,
                ..
            } => {
                *surf = surface;
                *swapchain = new_swapchain;
                *swapchain_images = new_images;
                *swapchain_textures = sc_textures;
            }
            RenderSurface::Offscreen { .. } => panic!(),
        };
    }

    pub fn offscreen(
        allocator: Arc<StandardMemoryAllocator>,
        dimensions: [u16; 2],
        pix_fmt: TexturePixelFormat,
    ) -> Result<Self, String> {
        Ok(Self::Offscreen {
            image: crate::texture::Texture::new(
                "Offscreen surface",
                [dimensions[0] as u32, dimensions[1] as u32, 1],
                false,
                TextureViewType::Dim2d,
                pix_fmt,
                TextureUseCase::Attachment,
                allocator,
            )?,
            dimensions: dimensions,
        })
    }
}

struct RenderableLight
{
    uniform_var: LightShaderStruct,
    shadow_map_mode: ShadowMapMode,
    static_shadow_buffer: Option<ShadowBuffer>,
    projections: Vec<ProjectionUniformData>,
    //static_shadow_framebuffers: Vec<(Framebuffer, ProjectionUniformData)>,
    refresh_flag: bool
}

/// Основная структура для рендеринга
pub struct Renderer {
    _command_buffer_father: CommandBufferFather,
    _allocator: Arc<BumpMemoryAllocator>,
    _ds_allocator: Arc<StandardDescriptorSetAllocator>,

    _context: Arc<Instance>,
    _vk_surface: RenderSurface,
    _device: Arc<Device>,
    _queue: Arc<Queue>,

    _frame_finish_event: Option<Box<dyn GpuFuture + 'static>>,
    _need_to_update_sc: bool,

    _draw_list: Vec<(GOTransform, Arc<MeshVisual>)>,
    _lights_list: Vec<RenderableLight>,

    _aspect: f32,
    _camera: Option<RcBox<GameObject>>,
    _camera_data: ProjectionUniformData,

    _geometry_pass: GeometryPass,

    _super_resolution: bool,
    _fxaa: bool,

    _postprocessor: PostprocessingPass,
    _shadowmap_pass: ShadowMapPass,

    _resource_manager: ResourceManagerRef,

    _dummy_texture_2d: Texture,
    _dummy_shadowmap: Texture,
    _dummy_texture_cube: Texture,

    _timer: UniformTime,
}

impl Renderer {
    pub fn postprocessor(&mut self) -> &mut PostprocessingPass {
        &mut self._postprocessor
    }

    pub fn render_result(&self) -> &Texture {
        match self._vk_surface {
            RenderSurface::Offscreen { ref image, .. } => image,
            RenderSurface::Winit {
                surface: _,
                swapchain: _,
                swapchain_images: _,
                ref swapchain_textures,
            } => &swapchain_textures[0],
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

    pub fn add_renderable_component(
        &mut self,
        transform_data: GOTransform,
        component: Arc<MeshVisual>,
    ) {
        self._draw_list.push((transform_data, component))
    }
}

use self::geometry_pass::check_in_frustum;
#[allow(dead_code)]
impl Renderer {
    pub fn default_device(
        vk_instance: Arc<Instance>,
    ) -> Result<(Arc<Device>, impl ExactSizeIterator<Item = Arc<Queue>>), Validated<vulkano::VulkanError>> {
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            khr_buffer_device_address: true,
            //khr_draw_indirect_count: true,
            ..DeviceExtensions::empty()
        };
        let (physical_device, queue_family_index) = vk_instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                // The Vulkan specs guarantee that a compliant implementation must provide at least one queue
                // that supports compute operations.
                p.queue_family_properties()
                    .iter()
                    .position(|q| !(q.queue_flags & QueueFlags::COMPUTE).is_empty())
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 1,
                PhysicalDeviceType::IntegratedGpu => 2,
                PhysicalDeviceType::VirtualGpu => 3,
                PhysicalDeviceType::Cpu => 4,
                PhysicalDeviceType::Other => 5,
                _ => 5,
            })
            .unwrap();
        let features = Features {
            sampler_anisotropy: true,
            //draw_indirect_count: true,
            multi_draw_indirect: true,
            depth_clamp: true,
            ..Features::empty()
        };
        let dev_info = DeviceCreateInfo {
            enabled_features: features,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: device_extensions, /*physical_device
                                                   .supported_extensions()
                                                   .union(&device_extensions),*/
            ..Default::default()
        };

        println!("Используется {}", physical_device.properties().device_name);
        //println!("{dev_info:?}");
        Device::new(physical_device, dev_info)
    }

    // pub fn offscreen(
    //     vk_instance: Arc<Instance>,
    //     dimensions: [u16; 2],
    //     super_resolution: bool,
    //     fxaa: bool,
    // ) -> Self {
    //     let (device, mut queues) = Self::default_device(vk_instance.clone()).unwrap();
    //     let queue = queues.next().unwrap();
    //     let allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    //     let command_buffer_father = CommandBufferFather::new(queue.clone());
    //     let surface = RenderSurface::offscreen(
    //         allocator.clone(),
    //         dimensions,
    //         TexturePixelFormat::B8G8R8A8_SRGB,
    //     )
    //     .unwrap();
    //     let result = Renderer {
    //         _vk_surface: surface,
    //         _aspect: dimensions[0] as f32 / dimensions[1] as f32,
    //         _context: vk_instance,
    //         _device: device.clone(),
    //         _queue: queue.clone(),
    //         _geometry_pass: GeometryPass::new(dimensions[0], dimensions[1], queue.clone()),
    //         _super_resolution: super_resolution,
    //         _fxaa: fxaa,
    //         _need_to_update_sc: true,
    //         _frame_finish_event: Some(sync::now(device.clone()).boxed()),
    //         _draw_list: Vec::new(),
    //         _lights_list: Vec::new(),

    //         _camera: None,
    //         _camera_data: ProjectionUniformData::default(),
    //         _postprocessor: PostprocessingPass::new(queue.clone(), dimensions[0], dimensions[1]),
    //         _shadowmap_pass: ShadowMapPass::new(queue.clone()),
    //         _timer: Default::default(),

    //         _dummy_texture_2d: Texture::new(
    //             "dummy_2d",
    //             [1, 1, 1],
    //             false,
    //             TextureViewType::Dim2d,
    //             TexturePixelFormat::R8G8B8A8_UINT,
    //             TextureUseCase::ReadOnly,
    //             allocator.clone()
    //         )
    //         .unwrap(),
    //         _dummy_shadowmap: Texture::new(
    //             "dummy_2d",
    //             [1, 1, 1],
    //             false,
    //             TextureViewType::Dim2d,
    //             TexturePixelFormat::D16_UNORM,
    //             TextureUseCase::ReadOnly,
    //             allocator.clone()
    //         )
    //         .unwrap(),
    //         _dummy_texture_cube: Texture::new(
    //             "dummy_cube",
    //             [1, 1, 6],
    //             false,
    //             TextureViewType::Dim2d,
    //             TexturePixelFormat::D16_UNORM,
    //             TextureUseCase::ReadOnly,
    //             allocator.clone()
    //         )
    //         .unwrap(),

    //         _allocator: allocator,
    //         _command_buffer_father: command_buffer_father,
    //         _ds_allocator: Arc::new(StandardDescriptorSetAllocator::new(device.clone(), StandardDescriptorSetAllocatorCreateInfo::default())),
    //     };
    //     result
    // }

    pub fn winit(
        vk_instance: Arc<Instance>,
        resource_manager: RcBox<ResourceManager>,
        surface: Arc<Surface>,
        dimensions: [u16; 2],
        vsync: bool,
        super_resolution: bool,
        fxaa: bool,
    ) -> Self {
        let _resource_manager = resource_manager.lock();
        let queue = _resource_manager.queue();
        let device = _resource_manager.device();
        let command_buffer_father = CommandBufferFather::new(queue.clone());
        let surface = RenderSurface::winit(surface, device.clone(), vsync).unwrap();

        let shadowmap_pass = ShadowMapPass::new(queue.clone());
        let geometry_pass = GeometryPass::new(dimensions[0], dimensions[1], &_resource_manager);
        let postprocessor = PostprocessingPass::new(queue.clone(), dimensions[0], dimensions[1]);

        let dummy_texture_2d = Texture::new(
            "dummy_2d",
            [1, 1, 1],
            false,
            TextureViewType::Dim2d,
            TexturePixelFormat::R8G8B8A8_UINT,
            TextureUseCase::ReadOnly,
            _resource_manager.allocator().clone()
        ).unwrap();

        let dummy_texture_cube = Texture::new(
            "dummy_cube",
            [1, 1, 1],
            false,
            TextureViewType::Cube,
            TexturePixelFormat::D16_UNORM,
            TextureUseCase::ReadOnly,
            _resource_manager.allocator().clone()
        ).unwrap();

        let result = Renderer {
            _vk_surface: surface,
            _aspect: dimensions[0] as f32 / dimensions[1] as f32,
            _context: vk_instance,
            _device: device.clone(),
            _queue: queue.clone(),
            _need_to_update_sc: true,
            _frame_finish_event: Some(sync::now(device.clone()).boxed()),

            _draw_list: Vec::new(),
            _lights_list: Vec::new(),

            _camera: None,
            _camera_data: ProjectionUniformData::default(),
            _shadowmap_pass: shadowmap_pass,
            _super_resolution: super_resolution,
            _fxaa: fxaa,
            _geometry_pass: geometry_pass,
            _postprocessor: postprocessor,
            _timer: Default::default(),

            _dummy_texture_2d: dummy_texture_2d.clone(),
            _dummy_shadowmap: dummy_texture_2d,
            _dummy_texture_cube: dummy_texture_cube,

            _resource_manager: resource_manager.clone(),

            _allocator: Arc::new(bump_memory_allocator_new_default(device.clone())),
            _command_buffer_father: command_buffer_father,
            _ds_allocator: Arc::new(StandardDescriptorSetAllocator::new(device.clone(), Default::default())),
        };
        result
    }

    fn resize(&mut self, width: u16, height: u16) -> Result<(), String>
    {
        let stsr = self._super_resolution;
        let gwidth = if stsr { width / 2 } else { width };
        let gheight = if stsr { height / 2 } else { height };
        self._geometry_pass = GeometryPass::new(gwidth, gheight, &self._resource_manager.lock());
        /* Создание узлов и связей графа постобработки */
        self._postprocessor.reset();
        
        let lighting = self._postprocessor.new_lighting(
            gwidth,
            gheight,
            MAX_SPOTLIGHTS,
            MAX_SUN_LIGHTS,
            MAX_POINT_LIGHTS,
            TexturePixelFormat::B8G8R8A8_SRGB,
        );
        let lighting = match lighting {
            Ok(st) => st,
            Err(err) => panic!("{err}"),
        };
        let ssr = self._postprocessor.new_ssr(gwidth, gheight)?;
        let composer = self._postprocessor.new_composing(gwidth, gheight)?;
        let denoiser = self._postprocessor.new_temporal_filter(width, height, stsr)?;

        let flip_y = self
            ._postprocessor
            .new_y_flip(
                width,
                height,
                TexturePixelFormat::B8G8R8A8_SRGB,
            )?;

        self._postprocessor.link_stages(lighting.stage_id, lighting.diffuse_out, None, ssr.stage_id, ssr.diffuse_in)?;
        self._postprocessor.link_stages(lighting.stage_id, lighting.specular_out, None, ssr.stage_id, ssr.specular_in)?;

        self._postprocessor.link_stages(ssr.stage_id, ssr.diffuse_out, None, composer.stage_id, composer.diffuse_input)?;
        self._postprocessor.link_stages(ssr.stage_id, ssr.specular_out, None, composer.stage_id, composer.specular_input)?;

        self._postprocessor.link_stages(composer.stage_id, composer.output, None, denoiser.stage_id, denoiser.input)?;
        

        if self._fxaa {
            let fxaa = self._postprocessor.new_fxaa(width, height).unwrap();
            self._postprocessor
                .link_stages(
                    denoiser.stage_id,
                    denoiser.output,
                    None,
                    fxaa.stage_id,
                    fxaa.input.to_owned(),
                )?;
            self._postprocessor
                .link_stages(
                    fxaa.stage_id,
                    fxaa.output,
                    None,
                    flip_y,
                    "image_in".to_owned(),
                )?;
        } else {
            self._postprocessor
                .link_stages(
                    denoiser.stage_id,
                    denoiser.output,
                    None,
                    flip_y,
                    "image_in".to_owned(),
                )?;
        }
        self._postprocessor
            .link_stages(flip_y, 0, None, 0, "swapchain_out".to_owned())?;

        //let ((_istage, _input), (ostage, output)) = self._postprocessor.fidelityfx_super_resolution(width, height);
        //self._postprocessor.link_stages(ostage, output, 0, "swapchain_out".to_owned());    // Соединение ноды с выходом.
        let self_camera = self._camera.clone();
        match self_camera {
            Some(ref camera) => {
                let mut _camera = camera.lock();
                _camera
                    .camera_mut()
                    .unwrap()
                    .set_aspect_dimenstions(width, height);
                let cam_component = _camera.camera().unwrap();
                self._camera_data = cam_component.uniform_data(&*_camera);
            }
            None => (),
        }
        Ok(())
    }

    /*pub fn width(&self) -> u16
    {
        self._vk_surface.dimensions()[0]
    }

    pub fn height(&self) -> u16
    {
        self._vk_surface.dimensions()[1]
    }*/

    pub fn swapchain_dims(&self) -> [u16; 2] {
        self._vk_surface.dimensions()
    }

    pub fn update_timer(&mut self, timer: UniformTime) {
        //self._timer = timer.clone();
        self._postprocessor.timer = timer;
        //self._postprocessor.uniform_to_all(&format!("timer"), timer);
    }

    pub fn update_camera_data(&mut self, camera: ProjectionUniformData) {
        self._camera_data = camera.clone();
    }

    /// Обновление swapchain изображений
    /// Как правило необходимо при изменении размера окна
    pub fn update_swapchain(&mut self, dimesions: Option<[u16; 2]>) {
        self._vk_surface.update_swapchain();
        let dimensions = match dimesions {
            Some(dims) => dims,
            None => self._vk_surface.dimensions(),
        };
        match self.resize(dimensions[0], dimensions[1]) {
            Ok(_) => (),
            Err(err) => panic!("{err}")
        };
    }

    /// Начинает проход геометрии
    pub fn begin_geametry_pass(&mut self) {
        self._draw_list.clear();
        self._lights_list.clear();
    }

    /// Передаёт объект для прохода геометрии
    pub fn draw(&mut self, obj: RcBox<GameObject>) {
        let owner = obj.lock();
        let owner_transform = owner.transform.clone();
        match owner.visual() {
            Some(visual) => {
                self.add_renderable_component(owner.transform().clone(), Arc::new(visual.clone()));
            }
            None => (),
        }
        match owner.light() {
            Some(_light) => {
                let mut light = _light.lock().unwrap();
                let in_frustum = check_in_frustum(&light.bbox_corners(), self._camera_data, owner_transform.global);
                let static_buffer = match light.static_shadow_buffer() {
                    Some(sb) => Some(sb.linked_copy()),
                    None => None
                };
                //if in_frustum
                {
                    let light = RenderableLight {
                        uniform_var: light.uniform_struct(-1),
                        static_shadow_buffer: static_buffer,
                        projections: light.projections(),
                        //static_shadow_framebuffers: light.static_shadow_framebuffers(),
                        refresh_flag: light.take_refresh_flag(),
                        shadow_map_mode: light.shadow_map_mode()
                    };
                    self._lights_list.push(light);
                }
            }
            None => (),
        }
        for child in owner.children().clone() {
            self.draw(child);
        }
    }

    pub fn wait(&mut self) {
        if let Some(mut future) = self._frame_finish_event.take() {
            future.flush().unwrap();
            future.cleanup_finished();
        }
        self._frame_finish_event = Some(sync::now(self.device().clone()).boxed());
    }

    /*fn build_lights_buffer(&mut self) -> (PrimaryAutoCommandBuffer, LightsUniformData)
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
            self._queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit
        ).unwrap();
        lccbb
            .update_data(&self._lights_data, light_data.to_vec()).unwrap();
        let light_compile_cb = lccbb.build().unwrap();
        (light_compile_cb, LightsUniformData::new(spotlights, point_lights, sun_lights))
    }*/

    /// Выполняет все сформированные буферы команд
    pub fn execute(
        &mut self,
        inputs: &HashMap<String, Texture>,
        resource_manager: &ResourceManagerRef,
    ) {
        self._frame_finish_event
            .as_mut()
            .unwrap()
            .cleanup_finished();

        if self._need_to_update_sc {
            self.update_swapchain(None);
            self._need_to_update_sc = false;
        }

        let (sc_target, image_num, suboptimal, acquire_future) =
            self._vk_surface.acquire_image().unwrap();
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

        // Отбор источников света
        let mut lights = std::mem::take(&mut self._lights_list);
        let camera_location: [f32; 16] = self._camera_data.transform.into();
        let camera_location = camera_location.vec3_location();
        lights.sort_by(|li1, li2| {
            let pos1 = li1.uniform_var.base().location().xyz();
            let pos2 = li2.uniform_var.base().location().xyz();
            let d1 = (camera_location - pos1).magnitude();
            let d2 = (camera_location - pos2).magnitude();
            match d1.partial_cmp(&d2) {
                Some(ord) => ord,
                None => Ordering::Equal,
            }
        });
        /*for (li, _) in &mut lights {
            resource_manager.lock().attach_shadow_buffer(li).unwrap();
        }
        resource_manager.lock().flush_futures();*/
        //let lights = lights[0..lights.len().min(16)].to_vec();
        //let lights = self._lights_list.clone();

        // Проход карт теней
        let mut sm_command_buffers: Vec<Arc<PrimaryAutoCommandBuffer>> = Vec::new();

        let static_objects = self
            ._draw_list
            .iter()
            .filter_map(|(transform, mesh_visual)| {
                if transform.is_static() {
                    Some((transform.uniform_value(), mesh_visual.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let dynamic_objects = self
            ._draw_list
            .iter()
            .filter_map(|(transform, mesh_visual)| {
                if !transform.is_static() {
                    Some((transform.uniform_value(), mesh_visual.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for li in &mut lights {
            /*if let ShadowMapMode::None = li.shadow_map_mode {
                li.framebuffers(owner_transform);
                return (li.clone(), -1, vec![], owner_transform.clone());
            };*/
            // Обновление статичной карты теней.
            //let mut _resour_manager = resource_manager.lock();
            if li.refresh_flag {
                let mut framebuffers = li.static_shadow_buffer.as_ref().unwrap().frame_buffers().clone();
                for (projection_data, framebuffer) in li.projections.iter_mut().zip(framebuffers.iter_mut()) {
                    let clear_pacb = self
                        ._command_buffer_father
                        .new_primary_instant(|pacbb| {
                            pacbb
                                .clear_depth_stencil(framebuffer.depth_attachment(), 1.0.into())
                                .unwrap();
                        }).unwrap().1;
                    let draw_pacb = self
                        ._shadowmap_pass
                        .build_shadow_map_pass(
                            framebuffer,
                            projection_data.clone(),
                            self._postprocessor.timer,
                            static_objects.clone(),
                            &self._command_buffer_father,
                            self._allocator.clone(),
                            self._ds_allocator.clone()
                        )
                        .unwrap();
                    sm_command_buffers.extend([clear_pacb, draw_pacb]);
                }
            }
            // Обновление динамической карты теней.
            let shadowmap_objects = match li.shadow_map_mode {
                ShadowMapMode::FullyDynamic(_) => {
                    [dynamic_objects.clone(), static_objects.clone()].concat()
                }
                ShadowMapMode::SemiDynamic(_) => dynamic_objects.clone(),
                _ => Vec::new(),
            };
            if let ShadowMapMode::None = li.shadow_map_mode {
                continue;
            }
            if let Some((sb, sbi)) = resource_manager
                .lock()
                .get_shadow_buffer_for_light(li.uniform_var.ty())
            {
                li.uniform_var.base_mut().set_shadow_map_index(sbi as _);
                let pacb = self
                    ._command_buffer_father
                    .new_primary_instant(|acbb| {
                        match li.shadow_map_mode {
                            ShadowMapMode::FullyDynamic(_) => {
                                // Заполнение динамического буфера для полностью динамических теней.
                                acbb.clear_depth_stencil(sb.buffer(), 1.0.into()).unwrap();
                            },
                            ShadowMapMode::SemiDynamic(_) | ShadowMapMode::Static(_) => {
                                // Копирование содержимого из буфера статичных теней.
                                acbb.copy_texture(li.static_shadow_buffer.as_ref().unwrap().buffer(), sb.buffer()).unwrap();
                            },
                            _ => ()
                        };
                    }
                ).unwrap().1;
                sm_command_buffers.push(pacb);
                let mut frame_buffers = sb.frame_buffers().clone();
                for (projection_data, shadow_framebuffer) in li.projections.iter_mut().zip(frame_buffers.iter_mut()) {
                    let command_buffer = self
                        ._shadowmap_pass
                        .build_shadow_map_pass(
                            shadow_framebuffer,
                            *projection_data,
                            self._postprocessor.timer,
                            shadowmap_objects.clone(),
                            &self._command_buffer_father,
                            self._allocator.clone(),
                            self._ds_allocator.clone()
                        )
                        .unwrap();
                    sm_command_buffers.push(command_buffer);
                }
            }
        }

        // Формирование uniform буфера с данными об источниках света.
        let spotlights = lights
            .iter()
            .filter_map(|li| {
                if let LightShaderStruct::Spot(light) = li.uniform_var {
                    Some(light)
                } else {
                    None
                }
            })
            .chain((lights.len()..MAX_SPOTLIGHTS as usize).map(|_| SpotlightUniform::default()))
            .collect::<Vec<_>>();

        let point_lights = lights
            .iter()
            .filter_map(|li| {
                if let LightShaderStruct::Point(light) = li.uniform_var {
                    Some(light)
                } else {
                    None
                }
            })
            .chain((lights.len()..MAX_POINT_LIGHTS as usize).map(|_| PointLightUniform::default()))
            .collect::<Vec<_>>();

        let sun_lights = lights
            .iter()
            .filter_map(|li| {
                if let LightShaderStruct::Sun(light) = li.uniform_var {
                    Some(light)
                } else {
                    None
                }
            })
            .chain((lights.len()..MAX_SUN_LIGHTS as usize).map(|_| SunLightUniform::default()))
            .collect::<Vec<_>>();
        
        self._postprocessor.uniform_array_to_all(&"ppSpotlights".to_owned(), &spotlights);
        self._postprocessor.uniform_array_to_all(&"ppSunlights".to_owned(), &sun_lights);
        self._postprocessor.uniform_array_to_all(&"ppPointlights".to_owned(), &point_lights);
        self._postprocessor.image_to_all("sun_shadowmaps", resource_manager.lock().sun_light_shadow_map_array());
        self._postprocessor.image_to_all("point_shadowmaps", resource_manager.lock().point_light_shadow_map_array());
        self._postprocessor.image_to_all("spot_shadowmaps", resource_manager.lock().spotlight_shadow_map_array());

        // Сборка буфера с информацией об источниках света
        //let (light_compile_cb, _light_count) = self.build_lights_buffer();
        let light_count = LightsUniformData::new(
            lights.iter().filter(|li| match li.uniform_var {LightShaderStruct::Spot(_) => true, _ => false}).count() as _,
            lights.iter().filter(|li| match li.uniform_var {LightShaderStruct::Point(_) => true, _ => false}).count() as _,
            lights.iter().filter(|li| match li.uniform_var {LightShaderStruct::Sun(_) => true, _ => false}).count() as _,
        );

        if let Some(ref camera) = self._camera {
            let cam_obj = camera.lock();
            let component_camera = cam_obj.camera().unwrap().clone();
            self._camera_data = component_camera.uniform_data(&*cam_obj);
        }

        // Построение прохода геометрии
        let gp_command_buffer = self
            ._geometry_pass
            .build_geometry_pass(
                self._camera_data,
                self._postprocessor.timer,
                [static_objects, dynamic_objects].concat(),
            )
            .unwrap();

        // Построение прохода постобработки
        // Передача входов в постобработку
        self._postprocessor
            .image_to_all(&"gAlbedo".to_owned(), self._geometry_pass.albedo());
        self._postprocessor
            .image_to_all(&"gNormals".to_owned(), self._geometry_pass.normals());
        self._postprocessor
            .image_to_all(&"gMasks".to_owned(), self._geometry_pass.specromet());
        self._postprocessor
            .image_to_all(&"gDepth".to_owned(), self._geometry_pass.depth());
        self._postprocessor
            .image_to_all(&"gVectors".to_owned(), self._geometry_pass.vectors());
        self._postprocessor
            .uniform_to_all(&"lights_count".to_owned(), light_count);
        self._postprocessor
            .uniform_to_all(&"camera".to_owned(), self._camera_data);

        if inputs.len() > 0 {
            for (name, img) in inputs {
                self._postprocessor.image_to_all(name, img);
            }
        }

        // Подключение swapchain-изображения в качестве выхода
        match self._vk_surface {
            RenderSurface::Winit { .. } => self
                ._postprocessor
                .set_output("swapchain_out".to_owned(), sc_target.clone()),
            RenderSurface::Offscreen { .. } => self
                ._postprocessor
                .set_output("swapchain_out".to_owned(), sc_target.clone()),
        }
        let pp_command_buffer = self._postprocessor.execute_graph();
        let _sc_out = self
            ._postprocessor
            .get_output("swapchain_out".to_owned())
            .unwrap();

        let pp_command_buffer = pp_command_buffer.build_buffer();

        let mut future = vulkano::sync::now(self._device.clone()).boxed();
        resource_manager.lock().flush_futures();
        self.wait();
        for cb in sm_command_buffers {
            future = future
                .then_execute(self._queue.clone(), cb)
                .unwrap()
                .boxed();
        }
        future = future
            .then_execute(self._queue.clone(), gp_command_buffer)
            .unwrap()
            .boxed()
            .boxed();

        match self._vk_surface {
            RenderSurface::Winit { .. } => {
                future = future.join(acquire_future.unwrap()).boxed();
            }
            _ => (),
        }
        future = future
            .then_execute_same_queue(pp_command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .boxed();

        match self._vk_surface {
            RenderSurface::Winit { ref swapchain, .. } => {
                let pi = SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_num as _);
                future = future
                    .then_swapchain_present(self._queue.clone(), pi)
                    .boxed();
            }
            _ => (),
        };
        let future = future.then_signal_fence_and_flush().map_err(Validated::unwrap);
        match future {
            Ok(future) => {
                self._frame_finish_event = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                self._need_to_update_sc = true;
                self._frame_finish_event = Some(sync::now(self._device.clone()).boxed());
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                self._frame_finish_event = Some(sync::now(self._device.clone()).boxed());
            }
        };
        resource_manager.lock().free_all_shadow_buffers();
    }

    pub fn queue(&self) -> &Arc<Queue> {
        &self._queue
    }

    pub fn device(&self) -> &Arc<Device> {
        &self._device
    }

    pub fn set_camera(&mut self, camera: RcBox<GameObject>) {
        if camera.lock().camera().is_some() {
            self._camera = Some(camera.clone());
        } else {
            panic!("Указанный объект не содержит компонента камеры");
        }
    }

    pub fn camera(&mut self) -> Option<GameObjectRef> {
        self._camera.clone()
    }
}

/// События
impl Renderer {
    fn check_object_in_frustum(
        projection_data: ProjectionUniformData,
        object_transform: GOTransformUniform,
        visual: Arc<MeshVisual>,
    ) -> (bool, bool, bool, bool, bool, bool, bool) {
        let projection: Mat4 = projection_data.projection.into();
        let view: Mat4 = projection_data.transform_inverted.into();
        let front = 0;
        let mut back = 0;
        let mut left = 0;
        let mut right = 0;
        let mut top = 0;
        let mut bottom = 0;
        let corners = visual.mesh().bbox_corners();
        let model = Mat4::from_iterator(object_transform.transform);
        let mvp_matrix = projection * view * model;
        for point in corners {
            let gl_position = mvp_matrix * Vec4::new(point.x, point.y, point.z, 1.0);
            let depth = gl_position.w;
            let normalized_pos = gl_position.xyz() / depth;
            if depth < 0.0 {
                back += 1;
                continue;
            }
            //if normalized_pos.z > 1.0  { front += 1; }
            if normalized_pos.x < -1.0 {
                left += 1;
            }
            if normalized_pos.x > 1.0 {
                right += 1;
            }
            if normalized_pos.y < -1.0 {
                bottom += 1;
            }
            if normalized_pos.y > 1.0 {
                top += 1;
            }
        }
        (
            front == 6 || back == 6 || left == 6 || right == 6 || top == 6 || bottom == 6,
            front == 6,
            back == 6,
            left == 6,
            right == 6,
            top == 6,
            bottom == 6,
        )
    }
}
