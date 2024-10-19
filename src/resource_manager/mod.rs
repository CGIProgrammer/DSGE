use std::{collections::HashMap, path::Path, sync::Arc};

use vulkano::{
    device::{Device, Queue},
    sync::GpuFuture, memory::allocator::StandardMemoryAllocator, image::sampler::SamplerMipmapMode,
};

use crate::{
    command_buffer::CommandBufferFather,
    components::light::{ShadowBuffer, LightType},
    material::{MaterialBuilder, MaterialRef},
    mesh::{Mesh, MeshRef, SubMesh},
    references::{MutexLockBox, RcBox},
    texture::{
        Texture, TextureRepeatMode, TexturePixelFormat, TextureFilter,
        TexturePixelFormatFeatures, TextureUseCase, TextureViewType
    },
};

pub const MAX_SPOTLIGHTS: u32 = 8;
pub const MAX_POINT_LIGHTS: u32 = 4;
pub const MAX_SUN_LIGHTS: u32 = 2;

#[derive(Clone, Copy, Debug)]
pub struct ResourceManagerConfig {
    pub max_spotlights: u32,
    pub max_sun_lights: u32,
    pub max_point_lights: u32,
    pub anisotrophy: Option<f32>,
    pub super_resolution: bool,
}

impl Default for ResourceManagerConfig {
    fn default() -> Self {
        Self {
            super_resolution: false,
            anisotrophy: None,
            max_spotlights: MAX_SPOTLIGHTS,
            max_sun_lights: MAX_SUN_LIGHTS,
            max_point_lights: MAX_POINT_LIGHTS
        }
    }
}

/// Менеджер ресурсов.
///
/// Управляет загрузкой ресурсов (текстуры, меши, материалы, шейдеры) и
/// некоторыми динамическими ресурсами.
pub struct ResourceManager {
    device: Arc<Device>,
    queue: Arc<Queue>,
    allocator: Arc<StandardMemoryAllocator>,
    command_buffer_father: CommandBufferFather,

    fs_path: String,
    textures_path: String,
    meshes_path: String,

    default_material: MaterialRef,

    materials: HashMap<String, MaterialRef>,
    textures: HashMap<String, Texture>,
    meshes: HashMap<String, MeshRef>,

    point_shadowmaps: DynamicShadowMapManager,
    spot_shadowmaps: DynamicShadowMapManager,
    sun_shadowmaps: DynamicShadowMapManager,

    config: ResourceManagerConfig,

    futures: Vec<Box<dyn GpuFuture>>,
}

pub type ResourceManagerRef = RcBox<ResourceManager>;

impl ResourceManager {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        config: ResourceManagerConfig,
    ) -> Result<Self, String> {
        let allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let command_buffer_father = CommandBufferFather::new(queue.clone());
        let (dt, _) = Texture::from_data(
            "dummy_2d",
            &[255, 255, 255, 255],
            [1, 1, 1],
            false, TextureViewType::Dim2d, 
            vulkano::format::Format::R8G8B8A8_SRGB,
            TextureUseCase::ReadOnly,
            &command_buffer_father,
            allocator.clone()
        )?;

        let mut material = MaterialBuilder::builder("default_material");
        material
            .add_numeric_parameter("diffuse", [0.8, 0.8, 0.8, 1.0].into())
            .add_numeric_parameter("roughness", 0.5.into())
            .add_numeric_parameter("glow", 0.0.into())
            .add_numeric_parameter("metallic", 0.0.into())
            .add_numeric_parameter("specular", 0.0.into())
            .add_texture("fDiffuseMap", &dt)
            .add_texture("fMetallicMap", &dt)
            .add_texture("fNornalMap", &dt)
            .add_texture("fRoughnessMap", &dt)
            .add_texture("fSpecularMap", &dt)
            .add_texture("fEmissionMap", &dt)
            .add_numeric_parameter("use_diffuse_map", 0.into())
            .add_numeric_parameter("use_metallic_map", 0.into())
            .add_numeric_parameter("use_normal_map", 0.into())
            .add_numeric_parameter("use_roughness_map", 0.into())
            .add_numeric_parameter("use_specular_map", 0.into())
            .add_numeric_parameter("use_emission_map", 0.into())
            .add_numeric_parameter("blend_method", 0.into())
            .add_numeric_parameter("shadow_method", 0.into());
        let material = material.build_mutex(device.clone(), config.super_resolution);

        Ok(Self {
            device: device.clone(),
            queue: queue.clone(),
            allocator: allocator.clone(),
            command_buffer_father: command_buffer_father,

            fs_path: "./data".to_owned(),
            textures_path: "textures".to_owned(),
            meshes_path: "mesh".to_owned(),
            config,

            default_material: material,

            materials: HashMap::new(),
            textures: HashMap::new(),
            meshes: HashMap::new(),
            
            point_shadowmaps: DynamicShadowMapManager::new(
                device.clone(),
                queue.clone(),
                256,
                256,
                config.max_point_lights,
                TexturePixelFormat::D16_UNORM,
                true,
            ),
            spot_shadowmaps: DynamicShadowMapManager::new(
                device.clone(),
                queue.clone(),
                512,
                512,
                config.max_spotlights,
                TexturePixelFormat::D16_UNORM,
                false,
            ),
            sun_shadowmaps: DynamicShadowMapManager::new(
                device.clone(),
                queue.clone(),
                1024,
                1024,
                config.max_sun_lights,
                TexturePixelFormat::D16_UNORM,
                false,
            ),

            futures: Vec::new(),
        })
    }

    pub fn command_buffer_father(&self) -> &CommandBufferFather
    {
        &self.command_buffer_father
    }

    pub fn allocator(&self) -> &Arc<StandardMemoryAllocator>
    {
        &self.allocator
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn queue(&self) -> &Arc<Queue> {
        &self.queue
    }

    pub fn config(&self) -> &ResourceManagerConfig {
        &self.config
    }

    pub fn new_material(&mut self, name: &str) -> MaterialRef {
        let mat = RcBox::construct(self.default_material.lock().fork(name));
        self.materials.insert(name.to_owned(), mat.clone());
        mat
    }

    pub fn get_material(&self, name: &str) -> Option<MaterialRef> {
        self.materials.get(name).cloned()
    }

    pub fn get_texture(&mut self, name: &str) -> Result<Texture, String> {
        // let _name = name.replace(".jpg", ".dds").replace(".png", ".dds");
        // let name = _name.as_str();
        if let Some(texture) = self.textures.get(name) {
            return Ok(texture.clone());
        }
        let fname = Path::new(self.fs_path.as_str())
            .join(self.textures_path.as_str())
            .join(name);
        if !fname.is_file() {
            return Err(format!("Файл {fname:?} не найден."));
        }
        let fname = fname.as_os_str().to_str().unwrap();
        let (mut texture, future) = Texture::from_file(&self.command_buffer_father, self.allocator.clone(), fname, true, true).unwrap();
        texture.set_mipmap_mode(SamplerMipmapMode::Linear);
        texture.set_mag_filter(TextureFilter::Linear);
        texture.set_min_filter(TextureFilter::Linear);
        texture.set_anisotropy(self.config.anisotrophy);
        self.textures.insert(name.to_owned(), texture.clone());
        self.futures.push(future);
        Ok(texture)
    }

    pub fn get_mesh(&mut self, name: &str) -> Option<MeshRef> {
        if let Some(mesh) = self.meshes.get(name) {
            return Some(mesh.clone());
        }
        let fname = Path::new(self.fs_path.as_str())
            .join(self.meshes_path.as_str())
            .join(name);
        if !fname.is_file() {
            return None;
        }
        let fname = fname.as_os_str().to_str().unwrap();
        let mut mesh = Mesh::builder(name);
        mesh.push_from_file(fname).unwrap();
        let mesh = mesh.build_mutex(
            &self.command_buffer_father,
            self.allocator.clone()
        ).unwrap();
        self.meshes.insert(name.to_owned(), mesh.clone());
        Some(mesh)
    }

    pub fn get_batch_of_meshes(&mut self, names: &[String]) -> HashMap<String, MeshRef> {
        let mesh_path = Path::new(self.fs_path.as_str()).join(self.meshes_path.as_str());
        let unloaded = names
            .into_iter()
            .filter_map(|name| {
                if self.meshes.contains_key(name) {
                    None
                } else {
                    Some(name)
                }
            })
            .collect::<Vec<_>>();

        let mut submeshes = HashMap::new();
        let mut mesh_builder = Mesh::builder("");

        for name in &unloaded {
            let fname = mesh_path.join(*name);
            if !fname.is_file() {
                panic!("Файл {fname:?} не найден.");
            }
            let submesh = mesh_builder
                .push_from_file(fname.to_str().unwrap())
                .unwrap();
            submeshes.insert(name, submesh);
        }
        println!("Сборка буфера полигональных сеток");
        let mesh_buffer = mesh_builder.build(&self.command_buffer_father, self.allocator.clone()).unwrap();
        println!("Сборка буфера завершена");

        for name in &unloaded {
            let (base, count, bbox) = submeshes[name];
            let submesh =
                SubMesh::from_mesh((*name).to_owned(), &mesh_buffer, bbox, base, count, 0);
            self.meshes.insert((*name).to_owned(), submesh);
        }

        names
            .into_iter()
            .map(|e| (e.clone(), self.meshes[e].clone()))
            .collect()
    }

    pub fn get_shadow_buffer_for_light(
        &mut self,
        light: LightType,
    ) -> Option<(
        ShadowBuffer,
        u32,
    )> {
        let shadow_buffer_pool = match light {
            LightType::Spot => &mut self.spot_shadowmaps,
            LightType::Sun => &mut self.sun_shadowmaps,
            LightType::Point => &mut self.point_shadowmaps,
        };
        return shadow_buffer_pool.get();
    }

    pub fn spotlight_shadow_map_array(&self) -> &Texture {
        &self.spot_shadowmaps.data
    }

    pub fn sun_light_shadow_map_array(&self) -> &Texture {
        &self.sun_shadowmaps.data
    }

    pub fn point_light_shadow_map_array(&self) -> &Texture {
        &self.point_shadowmaps.data
    }

    pub fn free_all_shadow_buffers(&mut self) {
        self.spot_shadowmaps.free_all();
        self.point_shadowmaps.free_all();
        self.sun_shadowmaps.free_all();
    }

    /*pub fn attach_shadow_buffer(&mut self, light: &mut Light) -> Result<(), String>
    {
        match light.shadow_map_mode() {
            crate::components::light::ShadowMapMode::Static(_) |
            crate::components::light::ShadowMapMode::SemiDynamic(_) |
            crate::components::light::ShadowMapMode::FullyDynamic(_) => (),
            crate::components::light::ShadowMapMode::None => return Ok(()),
        };
        let future = match light {
            Light::Point(point) => point.set_dynamic_shadow_buffer(Some(self.point_shadowmaps.get().unwrap().0), self.queue.clone())?,
            Light::Spot (spot) => spot.set_dynamic_shadow_buffer(Some(self.spot_shadowmaps.get().unwrap().0), self.queue.clone())?,
            Light::Sun  (sun) => sun.set_dynamic_shadow_buffer(Some(self.sun_shadowmaps.get().unwrap().0), self.queue.clone())?,
        };
        self.futures.push(future);
        Ok(())
    }

    pub fn detach_shadow_buffer(&mut self, light: &mut Light) -> bool
    {
        match light {
            Light::Point(point) =>
            if let Some(sb) = point.dynamic_shadow_buffer() {
                self.point_shadowmaps.free(sb.buffer().image_view().subresource_range().array_layers.start/6);
                true
            } else {
                false
            },
            Light::Spot (spot) => {
                println!("detach_shadow_buffer {:?}", spot.dynamic_shadow_buffer().is_some());
                if let Some(sb) = spot.dynamic_shadow_buffer() {
                    self.spot_shadowmaps.free(sb.buffer().image_view().subresource_range().array_layers.start);
                    true
                } else {
                    false
                }
            },
            Light::Sun  (sun) =>
                if let Some(sb) = sun.dynamic_shadow_buffer() {
                    self.sun_shadowmaps.free(sb.buffer().image_view().subresource_range().array_layers.start);
                    true
                } else {
                    false
                }
        }
    }*/

    pub fn flush_futures(&mut self) {
        while !self.futures.is_empty() {
            self.futures.pop().unwrap().flush().unwrap();
        }
        self.point_shadowmaps.flush_futures();
        self.spot_shadowmaps.flush_futures();
        self.sun_shadowmaps.flush_futures();
    }
}

struct DynamicShadowMapManager {
    data: Texture,
    free_layers: Vec<u32>,
    cube: bool,
    future: Option<Box<dyn GpuFuture>>,
}

impl DynamicShadowMapManager {
    fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        width: u32,
        height: u32,
        array_layers: u32,
        pix_fmt: TexturePixelFormat,
        cube: bool,
    ) -> Self {
        debug_assert!(
            pix_fmt.is_depth(),
            "Буфер глубины не может иметь формат {pix_fmt:?}"
        );
        let command_buffer_father = CommandBufferFather::new(queue.clone());
        let allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let cube_array_layers = if cube {
            array_layers * 6
        } else {
            array_layers
        };
        println!("Dynamic shadow buffer {width}, {height}, {cube_array_layers}\n");
        let mut data = Texture::new(
            "DynamicShadowMapManager",
            [width, height, cube_array_layers],
            false,
            TextureViewType::Dim2dArray,
            pix_fmt,
            TextureUseCase::Attachment,
            allocator.clone(),
        )
        .unwrap();
        data.set_address_mode([TextureRepeatMode::ClampToBorder, TextureRepeatMode::ClampToBorder, TextureRepeatMode::ClampToBorder]);
        data.clear_depth_stencil(&command_buffer_father, 1.0.into()).unwrap();
        let free_layers = (0..array_layers).collect();
        Self {
            data: data,
            free_layers,
            cube: cube,
            future: None,
        }
    }

    fn get(&mut self) -> Option<(ShadowBuffer, u32)> {
        if self.free_layers.is_empty() {
            return None;
        }
        let num = unsafe { self.free_layers.pop().unwrap_unchecked() };
        let sb = if self.cube {
            self.data
                .array_slice_as_texture((num * 6)..(num * 6 + 6))
                .unwrap()
        } else {
            self.data.array_layer_as_texture(num).unwrap()
        };
        Some((ShadowBuffer::from_texture(&sb), num))
    }

    fn layers(&self) -> u32 {
        if self.cube {
            self.data.array_layers() / 6
        } else {
            self.data.dims()[2]
        }
    }

    fn free_all(&mut self) {
        let layers = self.layers();
        self.free_layers = (0..layers).collect();
    }

    fn flush_futures(&mut self) {
        if let Some(mut future) = self.future.take() {
            future.flush().unwrap();
            future.cleanup_finished();
        }
    }
}
