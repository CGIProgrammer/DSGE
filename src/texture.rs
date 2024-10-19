mod dds_header;
mod ktx_header;
mod pixel_format;
mod types;

use crate::command_buffer::CommandBufferFather;
pub use crate::references::*;
pub use crate::shader::ShaderStructUniform;
use image::{imageops, DynamicImage};
pub use pixel_format::TexturePixelFormatFeatures;
pub use types::*;
use vulkano::device::DeviceOwned;
use vulkano::format::{ClearColorValue, ClearDepthStencilValue};
use vulkano::image::sampler::LOD_CLAMP_NONE;
use vulkano::image::{
    Image, ImageAspects, ImageCreateInfo, ImageSubresourceLayers, ImageSubresourceRange, ImageType,
};
use vulkano::DeviceSize;

use vulkano::memory::allocator::{
    AllocationCreateInfo, DeviceLayout, GenericMemoryAllocator, MemoryTypeFilter, StandardMemoryAllocator, Suballocator
};
use vulkano::memory::{DeviceAlignment, MemoryPropertyFlags};
use vulkano::pipeline::graphics::depth_stencil::CompareOp;

use std::ffi::c_void;
use std::fmt::Formatter;
use std::io::{BufRead, Read, Seek};
use std::num::NonZeroU64;
use std::ops::{Range, RangeInclusive};

#[cfg(feature = "use_image")]
use image::io::Reader as ImageReader;
#[cfg(feature = "use_image")]
use vulkano::command_buffer::CopyImageToBufferInfo;

#[allow(dead_code)]
use vulkano::device::Device;
//use vulkano::buffer::BufferContents;

#[allow(dead_code)]
use vulkano::image::view::{ImageView, ImageViewCreateInfo, ImageViewType};

pub use vulkano::image::view::ImageViewType as TextureViewType;
pub type TextureDimensions = [u32; 3];

use std::sync::Arc;
pub use vulkano::image::sampler::{
    Filter as TextureFilter, Sampler as TextureSampler, SamplerAddressMode as TextureRepeatMode,
    SamplerCreateInfo, SamplerMipmapMode as MipmapMode,
};
use vulkano::sync::GpuFuture;

use dds_header::DDSHeader;
use ktx_header::KTXHeader;
pub use pixel_format::TexturePixelFormat;

macro_rules! sampler_attribute {
    {$name:ident : $class:ty, $setter_name:ident} => {
        #[inline(always)]
        pub fn $name(&self) -> $class {
            self._vk_sampler.$name()
        }

        pub fn $setter_name(&mut self, value: $class) {
            let mut sampler_creeate_info = SamplerCreateInfo {
                address_mode : self._vk_sampler.address_mode(),
                mag_filter : self._vk_sampler.mag_filter(),
                min_filter : self._vk_sampler.min_filter(),
                mipmap_mode : self._vk_sampler.mipmap_mode(),
                mip_lod_bias : self._vk_sampler.mip_lod_bias(),
                anisotropy : self._vk_sampler.anisotropy(),
                lod : self._vk_sampler.lod(),
                compare : self._vk_sampler.compare(),
                ..Default::default()
            };
            sampler_creeate_info.$name = value;
            self._vk_sampler = TextureSampler::new(self._vk_device.clone(), sampler_creeate_info).unwrap();
        }
    };


    {$name:ident -> $inner_name:ident : $class:ty} => {
        #[inline(always)]
        fn $name(&self) -> &$class {
            &self._vk_sampler.$name
        }
        pub fn set_$name(&self, value: $class) {
            let mut sampler_creeate_info = SamplerCreateInfo {
                address_mode : self._vk_sampler.address_mode(),
                mag_filter : self._vk_sampler.mag_filter(),
                min_filter : self._vk_sampler.min_filter(),
                mipmap_mode : self._vk_sampler.mipmap_mode(),
                mip_lod_bias : self._vk_sampler.mip_lod_bias(),
                anisotropy : self._vk_sampler.anisotropy(),
                lod : self._vk_sampler.lod(),
                compare : self._vk_sampler.compare(),
                ..Default::default()
            };
            sampler.$inner_name = value;
            self._vk_sampler = TextureSampler::new(self._vk_device.clone(), sampler_creeate_info).unwrap();
        }
    };
}

//pub type TextureRef = RcBox<Texture>;

/// Текстура (она же изображение)
#[allow(dead_code)]
#[derive(Clone)]
pub struct Texture {
    name: String,

    pub(crate) _vk_image_dims: TextureDimensions,
    pub(crate) _vk_image_view: Arc<ImageView>,
    pub(crate) _vk_image_access: Arc<Image>,
    pub(crate) _vk_sampler: Arc<TextureSampler>,
    pub(crate) _vk_device: Arc<Device>,

    _pix_fmt: TexturePixelFormat,

    min_filter: TextureFilter,
    mag_filter: TextureFilter,
    mip_mode: MipmapMode,
    u_repeat: TextureRepeatMode,
    v_repeat: TextureRepeatMode,
    w_repeat: TextureRepeatMode,
    mip_lod_bias: f32,
    max_anisotropy: Option<f32>,
    compare_op: Option<CompareOp>,
    min_lod: f32,
    max_lod: f32,
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("Texture")
            .field("_vk_image_dims", &self._vk_image_dims)
            .field("_pix_fmt", &self._pix_fmt)
            .field("min_filter", &self.min_filter)
            .field("mip_mode", &self.mip_mode)
            .field("u_repeat", &self.u_repeat)
            .field("v_repeat", &self.v_repeat)
            .field("w_repeat", &self.w_repeat)
            .field("max_anisotropy", &self.max_anisotropy)
            .field("min_lod", &self.min_lod)
            .field("max_lod", &self.max_lod);
        Ok(())
    }
}

use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, BlitImageInfo, BufferImageCopy, ClearColorImageInfo,
    ClearDepthStencilImageInfo, CopyBufferToImageInfo, ImageBlit,
};
use vulkano::image::{ImageCreateFlags, ImageLayout, ImageUsage};

#[allow(dead_code)]
impl Texture {
    pub fn box_id(&self) -> u32 {
        self._vk_sampler.as_ref() as *const TextureSampler as u32
            ^ self._vk_image_access.as_ref() as *const Image as *const c_void as u32
            ^ self._vk_image_view.as_ref() as *const ImageView as *const c_void as u32
    }

    pub fn new<A>(
        name: &str,
        dims: TextureDimensions,
        mipmaps: bool,
        ty: TextureViewType,
        pix_fmt: TexturePixelFormat,
        use_case: TextureUseCase,
        allocator: Arc<GenericMemoryAllocator<A>>,
    ) -> Result<Self, String> 
    where A: Suballocator + Send + 'static
    {
        // let size = dims[0] * dims[1] * dims[2] * pix_fmt.block_size() as u32 / pix_fmt.texels_per_block() as u32;
        let (flags, buffer_type, array_layers, extent) = match ty {
            TextureViewType::Dim1d => (
                ImageCreateFlags::default(),
                ImageType::Dim1d,
                1,
                [dims[0], 1, 1],
            ),
            TextureViewType::Dim1dArray => (
                ImageCreateFlags::default(),
                ImageType::Dim2d,
                dims[1],
                [dims[0], 1, 1],
            ),
            TextureViewType::Dim2d => (
                ImageCreateFlags::default(),
                ImageType::Dim2d,
                1,
                [dims[0], dims[1], 1],
            ),
            TextureViewType::Dim2dArray => (
                ImageCreateFlags::default(),
                ImageType::Dim2d,
                dims[2],
                [dims[0], dims[1], 1],
            ),
            TextureViewType::Dim3d => (
                ImageCreateFlags::default(),
                ImageType::Dim3d,
                1,
                [dims[0], dims[1], dims[2]],
            ),
            TextureViewType::Cube => (
                ImageCreateFlags::CUBE_COMPATIBLE,
                ImageType::Dim2d,
                6,
                [dims[0], dims[1], 1],
            ),
            TextureViewType::CubeArray => (
                ImageCreateFlags::CUBE_COMPATIBLE,
                ImageType::Dim2d,
                6 * dims[2],
                [dims[0], dims[1], 1],
            ),
            _ => unimplemented!(),
        };
        let mip_levels = if mipmaps {
            (dims[0].max(dims[1]) as f32).log2() as u32 + 1
        } else {
            1u32
        };

        let all_usage = ImageUsage::TRANSFER_SRC
            | ImageUsage::TRANSFER_DST
            | ImageUsage::STORAGE
            | ImageUsage::COLOR_ATTACHMENT
            | ImageUsage::DEPTH_STENCIL_ATTACHMENT;
        let (_layout, usage) = match (use_case, pix_fmt.is_depth()) {
            (TextureUseCase::General, _) => (ImageLayout::General, ImageUsage::TRANSFER_SRC | ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED),
            (TextureUseCase::Attachment, false) => (
                ImageLayout::ColorAttachmentOptimal,
                ImageUsage::SAMPLED | ImageUsage::COLOR_ATTACHMENT,
            ),
            (TextureUseCase::Attachment, true) => (
                ImageLayout::DepthStencilAttachmentOptimal,
                ImageUsage::SAMPLED
                    | ImageUsage::DEPTH_STENCIL_ATTACHMENT
                    | ImageUsage::TRANSFER_DST
                    | ImageUsage::TRANSFER_SRC,
            ),
            (TextureUseCase::ReadOnly, _) => (
                ImageLayout::DepthStencilAttachmentOptimal,
                ImageUsage::SAMPLED | ImageUsage::TRANSFER_DST,
            ),
            (TextureUseCase::Storage, _) => (ImageLayout::General, all_usage),
            // _ => (TransferSrcOptimal, ImageUsage::TRANSFER_SRC),
            // _ => (TransferDstOptimal, ImageUsage::TRANSFER_DST),
            // _ => (DepthReadOnlyStencilAttachmentOptimal, ImageUsage::DEPTH_STENCIL_ATTACHMENT),
            // _ => (DepthAttachmentStencilReadOnlyOptimal, ImageUsage::DEPTH_STENCIL_ATTACHMENT),
            // _ => (DepthAttachmentOptimal, ImageUsage::DEPTH_STENCIL_ATTACHMENT),
            // _ => (StencilAttachmentOptimal, ImageUsage::DEPTH_STENCIL_ATTACHMENT),
            // (_, _) => (ImageLayout::General, ImageUsage::SAMPLED),
        };
        let create_info = ImageCreateInfo {
            flags,
            image_type: buffer_type,
            format: pix_fmt,
            extent: extent,
            array_layers: array_layers,
            mip_levels: mip_levels,
            usage: usage,
            //initial_layout: layout,
            ..Default::default()
        };

        let allocation_info = AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter {
                required_flags: MemoryPropertyFlags::DEVICE_LOCAL,
                not_preferred_flags: MemoryPropertyFlags::DEVICE_COHERENT
                    | MemoryPropertyFlags::HOST_CACHED
                    | MemoryPropertyFlags::HOST_COHERENT
                    | MemoryPropertyFlags::DEVICE_COHERENT,
                ..Default::default()
            },
            ..Default::default()
        };
        let image = Image::new(allocator.clone(), create_info, allocation_info)
            .map_err(|e| e.unwrap().to_string())?;

        //let image_view = ImageView::new_default(image.clone()).unwrap();
        let ivci = ImageViewCreateInfo {
            view_type: ty,
            ..ImageViewCreateInfo::from_image(&image)
        };
        let image_view = ImageView::new(image.clone(), ivci).unwrap();
        let sampler = TextureSampler::new(
            allocator.device().clone(),
            SamplerCreateInfo {
                mag_filter: TextureFilter::Linear,
                min_filter: TextureFilter::Linear,
                mipmap_mode: MipmapMode::Linear,
                address_mode: [TextureRepeatMode::Repeat; 3],
                mip_lod_bias: 0.0,
                anisotropy: None,
                lod: 0.0..=LOD_CLAMP_NONE,
                ..Default::default()
            },
        )
        .map_err(|e| e.unwrap().to_string())?;

        Ok(Self {
            name: name.to_owned(),
            _vk_image_dims: extent,
            _vk_image_view: image_view,
            _vk_image_access: image,
            _vk_sampler: sampler.clone(),
            _vk_device: allocator.device().clone(),
            _pix_fmt: pix_fmt,
            min_filter: sampler.min_filter(),
            mag_filter: sampler.mag_filter(),
            mip_mode: sampler.mipmap_mode(),
            u_repeat: sampler.address_mode()[0],
            v_repeat: sampler.address_mode()[1],
            w_repeat: sampler.address_mode()[2],
            mip_lod_bias: sampler.mip_lod_bias(),
            max_anisotropy: sampler.anisotropy(),
            compare_op: sampler.compare(),
            min_lod: *sampler.lod().start(),
            max_lod: *sampler.lod().end(),
        })
    }

    fn raw_from_file<R>(
        name: &str,
        reader: &mut R,
        mipmaps: bool,
        srgb: bool,
        use_case: TextureUseCase,
        command_buffer_father: &CommandBufferFather,
        allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<(Self, Box<dyn GpuFuture>), String>
    where
        R: Read + Seek + BufRead,
    {
        let img_rdr = ImageReader::new(reader).with_guessed_format();
        if img_rdr.is_err() {
            return Err(String::from("Неизвестный формат изображения"));
        }
        let img_rdr = img_rdr.unwrap();
        let image = img_rdr.decode().unwrap();

        let (pix_fmt, (width, height)) = match &image {
            image::DynamicImage::ImageLuma8(img) => (
                //img.as_raw().as_bytes().to_vec(),
                TexturePixelFormat::R8_UNORM,
                img.dimensions(),
            ),
            image::DynamicImage::ImageLumaA8(img) => {
                (TexturePixelFormat::R8G8_UNORM, img.dimensions())
            }
            image::DynamicImage::ImageRgb8(img) => {(
                if srgb {TexturePixelFormat::R8G8B8A8_SRGB} else {TexturePixelFormat::R8G8B8A8_UNORM},
                img.dimensions()
            )}
            image::DynamicImage::ImageRgba8(img) => {(
                if srgb {TexturePixelFormat::R8G8B8A8_SRGB} else {TexturePixelFormat::R8G8B8A8_UNORM},
                img.dimensions()
            )}
            image::DynamicImage::ImageLuma16(img) => {
                (TexturePixelFormat::R16_UNORM, img.dimensions())
            }
            image::DynamicImage::ImageLumaA16(img) => {
                (TexturePixelFormat::R16G16_UNORM, img.dimensions())
            }
            image::DynamicImage::ImageRgb16(img) => {
                (TexturePixelFormat::R16G16B16_UNORM, img.dimensions())
            }
            image::DynamicImage::ImageRgba16(img) => {
                (TexturePixelFormat::R16G16B16A16_UNORM, img.dimensions())
            }
            image::DynamicImage::ImageRgb32F(img) => {
                (TexturePixelFormat::R32G32B32_SFLOAT, img.dimensions())
            }
            image::DynamicImage::ImageRgba32F(img) => {
                (TexturePixelFormat::R32G32B32A32_SFLOAT, img.dimensions())
            }
            _ => panic!("Неизвестный формат пикселей"),
        };
        let image = if let image::DynamicImage::ImageRgb8(_) = image {
            DynamicImage::ImageRgba8(image.to_rgba8())
        } else {
            image
        };
        let dims = [width, height, 1];
        let texture = Texture::new(
            name,
            dims,
            mipmaps,
            TextureViewType::Dim2d,
            pix_fmt,
            use_case,
            allocator.clone(),
        )?;
        let future = command_buffer_father
            .execute_in_new_primary(None, |pacbb| {
                for mip_level in 0..texture.lod_levels() {
                    let _image = if mip_level > 0 {
                        image.resize_exact(
                            1.max(width >> mip_level),
                            1.max(height >> mip_level),
                            imageops::FilterType::Lanczos3
                        )
                    } else {
                        image.clone()
                    };
                    if let Err(e) = pacbb.update_data(
                        allocator.clone(),
                        &texture,
                        _image.as_bytes(),
                        mip_level,
                        0
                    ) {
                        println!("{e}");
                        //println!("{}, {}x{} => {}x{}", &e[e.find("VUID").unwrap()..], width >> mip_level, height >> mip_level, _image.width(), _image.height());
                    };
                }
            })?
            .1;
        Ok((texture, future))
    }

    fn compressed_from_file<R>(
        name: &str,
        mipmaps: bool,
        reader: &mut R,
        command_buffer_father: &CommandBufferFather,
        allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<(Self, Box<dyn GpuFuture>), String>
    where
        R: Read + Seek + BufRead,
    {
        let header: Box<dyn CompressedFormat>;
        let mut header_bytes = vec![0u8; 128];
        reader.read(header_bytes.as_mut_slice()).unwrap();
        match DDSHeader::from_bytes(header_bytes.as_slice()) {
            Ok(hdr) => header = Box::new(hdr),
            Err(dds_error) => match KTXHeader::from_bytes(header_bytes.as_slice()) {
                Ok(hdr) => header = Box::new(hdr),
                Err(ktx_error) => return Err(format!("{}; {}", dds_error, ktx_error)),
            },
        };

        reader
            .seek(std::io::SeekFrom::Start(header.header_size() as u64))
            .unwrap();

        let mut width = header.dimensions().0;
        let mut height = header.dimensions().1;
        let mut data_size = 0;
        let dims = [width, height, 1];
        let pix_fmt = header.pixel_format();
        let block_size = header.block_size();
        let mip_levels = if mipmaps{
            header.mip_levels()
        } else {
            1
        };
        let texture = Self::new(
            name,
            dims,
            mipmaps,
            TextureViewType::Dim2d,
            pix_fmt,
            TextureUseCase::ReadOnly,
            allocator.clone(),
        )?;

        let mip0_blocks = ((width + 3) / 4) * ((height + 3) / 4);
        let mut lod_data_buffer = vec![0; block_size * mip0_blocks as usize];
        let future = command_buffer_father.execute_in_new_primary(None, |pcbb| {
            for lod_level in 0..mip_levels {
                let blocks = ((width + 3) / 4) * ((height + 3) / 4);
                let size: usize = blocks as usize * block_size;
                reader.read(&mut lod_data_buffer[0..size]).unwrap();
                data_size += size;
                width = 1.max(width / 2);
                height = 1.max(height / 2);
                pcbb.update_data(
                    allocator.clone(),
                    &texture,
                    &lod_data_buffer[0..size],
                    lod_level,
                    0,
                )
                .unwrap();
            }
        })?;
        Ok((texture, future.1))
    }

    pub fn from_data(
        name: &str,
        data: &[u8],
        dims: TextureDimensions,
        mipmaps: bool,
        ty: TextureViewType,
        pix_fmt: TexturePixelFormat,
        use_case: TextureUseCase,
        command_buffer_father: &CommandBufferFather,
        allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<(Self, Box<dyn GpuFuture>), String> {
        let texture = Self::new(
            name,
            dims,
            mipmaps,
            ty,
            pix_fmt,
            use_case,
            allocator.clone(),
        )?;
        let (_, future) = command_buffer_father.execute_in_new_primary(None, |pcbb| {
            pcbb.update_data(allocator, &texture, data, 0, 0).unwrap();
        })?;
        Ok((texture, future))
    }

    #[inline(always)]
    pub fn name(&self) -> &String {
        &self.name
    }

    #[inline(always)]
    pub fn ty(&self) -> TextureViewType {
        self._vk_image_view.view_type()
    }

    pub fn array_layer_as_texture(&self, layer: u32) -> Result<Texture, String> {
        let view_type: ImageViewType = match self.image_view().view_type() {
            ImageViewType::Dim1d => ImageViewType::Dim1d,
            ImageViewType::Dim1dArray => ImageViewType::Dim1d,
            ImageViewType::Dim2d => ImageViewType::Dim2d,
            ImageViewType::Dim2dArray => ImageViewType::Dim2d,
            ImageViewType::Cube => ImageViewType::Dim2d,
            ImageViewType::CubeArray => ImageViewType::Cube,
            other => return Err(format!("{other:?} не является массивом")),
        };
        let subresource_range = self._vk_image_view.subresource_range();
        Texture::from_vk_image_view(
            ImageView::new(
                self._vk_image_access.clone(),
                ImageViewCreateInfo {
                    view_type: view_type,
                    subresource_range: ImageSubresourceRange {
                        aspects: subresource_range.aspects,
                        array_layers: layer..(layer + 1),
                        mip_levels: 0..1,
                    },
                    format: self._pix_fmt,
                    ..Default::default()
                },
            )
            .unwrap(),
            self._vk_device.clone(),
        )
    }

    pub fn array_slice_as_texture(&self, layers: Range<u32>) -> Result<Texture, String> {
        let view_type = match (layers.end - layers.start, self.image_view().view_type()) {
            (1, ImageViewType::Dim1dArray) => ImageViewType::Dim1d,
            (1, ImageViewType::Dim2dArray) => ImageViewType::Dim2d,
            (_, view_type) => view_type,
        };
        let subresource_range = self._vk_image_view.subresource_range();
        Texture::from_vk_image_view(
            ImageView::new(
                self._vk_image_access.clone(),
                ImageViewCreateInfo {
                    view_type: view_type,
                    subresource_range: ImageSubresourceRange {
                        aspects: subresource_range.aspects,
                        array_layers: layers,
                        mip_levels: 0..1,
                    },
                    format: self._pix_fmt,
                    ..Default::default()
                },
            )
            .unwrap(),
            self._vk_device.clone(),
        )
    }

    /// Получает формат пикселя текстуры
    #[inline(always)]
    pub fn pix_fmt(&self) -> TexturePixelFormat {
        self._pix_fmt
    }

    pub fn load_data<P: AsRef<std::path::Path> + ToString>(
        &self,
        command_buffer_father: &CommandBufferFather,
        allocator: Arc<StandardMemoryAllocator>,
        _path: P,
    ) -> Result<(), String> {
        #[cfg(feature = "use_image")]
        {
            let extension = _path.as_ref().extension();
            //let mut texture_builder = Texture::builder();
            //texture_builder.name(path.to_owned().as_str());
            return match extension {
                None => Err(String::from("Неизвестный формат изображения")),
                Some(os_str) => {
                    let reader = std::fs::File::open(_path.as_ref());
                    if reader.is_err() {
                        return Err(format!("Файл {} не найден.", _path.to_string()));
                    }
                    let reader = reader.unwrap();
                    let buf_reader = std::io::BufReader::new(reader);
                    let img_rdr = ImageReader::new(buf_reader).with_guessed_format();
                    if img_rdr.is_err() {
                        return Err(String::from("Неизвестный формат изображения"));
                    }
                    let img_rdr = img_rdr.unwrap();
                    let image = img_rdr.decode().unwrap().to_rgba8();

                    match os_str.to_str() {
                        Some("dds") | Some("ktx") => {
                            Err("load_data не поддерживает обновление сжатых текстур".to_owned())
                        }
                        _ => {
                            let data = image.as_raw().clone();
                            command_buffer_father.execute_in_new_primary(None, |cbb| {
                                cbb.update_data(allocator, self, &data, 0, 0).unwrap();
                            })?;
                            Ok(())
                        }
                    }
                }
            };
        }
        #[cfg(not(feature = "use_image"))]
        {
            Err("Поддержка обычных изображений отключена.".to_owned())
        }
    }

    /// Сохраняет текстуру в синхронном режиме (после возврата результата).
    pub fn save<P: AsRef<std::path::Path> + ToString>(
        &self,
        command_buffer_father: &CommandBufferFather,
        allocator: Arc<StandardMemoryAllocator>,
        _path: P,
    ) -> Result<(), String> {
        #[cfg(feature = "use_image")]
        {
            //if self._pix_fmt.compression().is_some() || self._pix_fmt.block_extent() != [1,1,1] {
            //    panic!("Сохранение сжатых текстур не поддерживается. Да и зачем оно вообще нужно?");
            //}
            let subpix_count = self._pix_fmt.subpixels();
            let block_size = self._pix_fmt.block_size() as u32;
            let pix_fmt = match subpix_count {
                1 => TexturePixelFormat::R8G8B8A8_SRGB,
                2 => TexturePixelFormat::R8G8B8A8_SRGB,
                3 => TexturePixelFormat::R8G8B8A8_SRGB,
                4 => TexturePixelFormat::R8G8B8A8_SRGB,
                _ => panic!("Текстуры с {} компонентами не поддерживаются", subpix_count),
            };
            let img = Texture::new(
                "convertion_texture",
                self._vk_image_dims,
                false,
                self._vk_image_view.view_type(),
                pix_fmt,
                TextureUseCase::General,
                allocator.clone(),
            )?;
            let initial_data = (0..(block_size * self._vk_image_dims[0] * self._vk_image_dims[1] * self._vk_image_dims[2])).map(|_| 0u8).into_iter();
            let cpuab = /*unsafe*/ {
                Buffer::from_iter(
                    allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::TRANSFER_DST,
                        //size: (block_size * self._vk_image_dims[0] * self._vk_image_dims[1] * self._vk_image_dims[2]) as _,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::HOST_RANDOM_ACCESS | MemoryTypeFilter::PREFER_HOST,
                        ..Default::default()
                    },
                    //DeviceLayout::new(NonZeroU64::new(1).unwrap(), DeviceAlignment::MIN).unwrap(),
                    initial_data
                    
                ).unwrap()
            };
            println!(
                "Создан новый временный буфер текстуры размером {} байт",
                cpuab.size()
            );
            {
                command_buffer_father.execute_in_new_primary(None, |cbb| {
                    cbb.copy_texture(self, &img)
                        .unwrap()
                        .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
                            img._vk_image_access.clone(),
                            cpuab.clone(),
                        ))
                        .unwrap();
                })?;
            }

            let buf = cpuab.read().unwrap().to_vec();
            match subpix_count {
                1 => {
                    let img = image::GrayImage::from_raw(img.width(), img.height(), buf).unwrap();
                    img.save(_path).unwrap();
                }
                2 => {
                    let img =
                        image::GrayAlphaImage::from_raw(img.width(), img.height(), buf).unwrap();
                    img.save(_path).unwrap();
                }
                3 => {
                    let img = image::RgbImage::from_raw(img.width(), img.height(), buf).unwrap();
                    img.save(_path).unwrap();
                }
                4 => {
                    let img = image::RgbaImage::from_raw(img.width(), img.height(), buf).unwrap();
                    img.save(_path).unwrap();
                }
                _ => panic!(),
            };
            Ok(())
        }
        #[cfg(not(feature = "use_image"))]
        {
            panic!("Поддержка форматов изображений кроме dds и ktx отключена.");
        }
    }

    /// Создаёт `Texture` на основе `ImageViewAbstract`.
    /// В основном используется для представления swapchain изображения в виде текстуры
    /// для вывода результата рендеринга
    pub fn from_vk_image_view(img: Arc<ImageView>, device: Arc<Device>) -> Result<Texture, String> {
        let img_dims = img.image().extent();
        let sampler = TextureSampler::new(
            device.clone(),
            SamplerCreateInfo {
                address_mode: [
                    TextureRepeatMode::ClampToEdge,
                    TextureRepeatMode::ClampToEdge,
                    TextureRepeatMode::ClampToEdge,
                ],
                mipmap_mode: MipmapMode::Nearest,
                anisotropy: None,
                min_filter: TextureFilter::Nearest,
                mag_filter: TextureFilter::Nearest,
                ..Default::default()
            },
        )
        .unwrap();

        let pix_fmt = TexturePixelFormat::from_vk_format(img.image().format())?;
        //println!("from_vk_image_view: {}x{}", img_dims.width(), img_dims.height());

        //TextureDimensions{width: img_dims[0], }
        Ok(Self {
            name: "".to_owned(),

            _vk_image_dims: img_dims,
            _vk_image_access: img.image().clone(),
            _vk_image_view: img.clone(),
            _vk_sampler: sampler,
            _vk_device: device.clone(),

            _pix_fmt: pix_fmt,

            min_filter: TextureFilter::Nearest,
            mag_filter: TextureFilter::Nearest,
            mip_mode: MipmapMode::Nearest,
            u_repeat: TextureRepeatMode::Repeat,
            v_repeat: TextureRepeatMode::Repeat,
            w_repeat: TextureRepeatMode::Repeat,
            mip_lod_bias: 0.0,
            max_anisotropy: None,
            compare_op: None,
            min_lod: 0.0,
            max_lod: 1.0,
        })
    }

    /// Загрузка изображения-текстуры из файла.
    /// Поддерживаются форматы dds, ktx и все форматы, поддерживаемые crate'ом image
    pub fn from_file<P>(
        command_buffer_father: &CommandBufferFather,
        allocator: Arc<StandardMemoryAllocator>,
        path: P,
        srgb: bool,
        mipmaps: bool
    ) -> Result<(Texture, Box<dyn GpuFuture>), String>
    where
        P: AsRef<std::path::Path> + ToString,
    {
        let extension = path.as_ref().extension();
        /*let mut texture_builder = Texture::builder()
        .min_filter(TextureFilter::Linear)
        .mag_filter(TextureFilter::Linear)
        .vertical_address(TextureRepeatMode::Repeat)
        .horizontal_address(TextureRepeatMode::Repeat)
        .mipmap(MipmapMode::Linear)
        .need_mipmaps(true)
        .read_only()
        .name(path.to_string().as_str());*/

        match extension {
            None => return Err(String::from("Неизвестный формат изображения")),
            Some(os_str) => {
                let reader = std::fs::File::open(path.as_ref());
                if reader.is_err() {
                    return Err(format!("Файл {} не найден.", path.to_string()));
                }
                let reader = reader.unwrap();
                let mut buf_reader = std::io::BufReader::new(reader);
                match os_str.to_str() {
                    Some("dds") | Some("ktx") => Self::compressed_from_file(
                        &path.to_string(),
                        mipmaps,
                        &mut buf_reader,
                        command_buffer_father,
                        allocator,
                    ),
                    _ => {
                        //panic!("Загрузка текстур форматов кроме dds и ktx отключена");
                        Self::raw_from_file(
                            &path.to_string(),
                            &mut buf_reader,
                            mipmaps,
                            srgb,
                            TextureUseCase::ReadOnly,
                            command_buffer_father,
                            allocator,
                        )
                    }
                }
            }
        }
    }

    pub fn as_cubemap(&self) -> Result<Texture, String> {
        if self.dims()[2] > 0 && self.dims()[2] % 6 != 0 {
            return Err("Из этого буфера не получится сделать кубическую текстуру: количество слоёв массива не кратно 6".to_owned());
        }
        let mut ivci = ImageViewCreateInfo::from_image(self._vk_image_access.as_ref());
        ivci.view_type = ImageViewType::Cube;
        let iw = ImageView::new(self._vk_image_access.clone(), ivci).unwrap();
        Ok(Texture::from_vk_image_view(iw, self._vk_device.clone()).unwrap())
    }

    /// Возвращает представление изображения
    #[inline(always)]
    pub fn image_view(&self) -> &Arc<ImageView> {
        &self._vk_image_view
    }

    /// Возвращает сэмплер изображения
    pub fn sampler(&self) -> &Arc<TextureSampler> {
        &self._vk_sampler
    }

    sampler_attribute! {address_mode: [TextureRepeatMode; 3], set_address_mode}
    sampler_attribute! {mipmap_mode: MipmapMode, set_mipmap_mode}
    sampler_attribute! {anisotropy: Option<f32>, set_anisotropy}
    sampler_attribute! {min_filter: TextureFilter, set_min_filter}
    sampler_attribute! {mag_filter: TextureFilter, set_mag_filter}
    sampler_attribute! {lod: RangeInclusive<f32>, set_lod}

    /// Ширина (длина для 1D текстур)
    pub fn width(&self) -> u32 {
        self._vk_image_dims[0]
    }

    /// Высота (1 писель для 1D текстур)
    pub fn height(&self) -> u32 {
        self._vk_image_dims[1]
    }

    /// Глубина (1 пиксель для 1D и 2D текстур)
    pub fn depth(&self) -> u32 {
        self._vk_image_dims[2]
    }

    pub fn array_layers(&self) -> u32 {
        let subresource = self._vk_image_view.subresource_range().clone();
        subresource.array_layers.count() as _
    }

    pub fn lod_levels(&self) -> u32 {
        self._vk_image_access.mip_levels()
        //self.width().max(self.height()).max(self.depth()).ilog2() + 1
    }

    pub fn dims(&self) -> TextureDimensions {
        self._vk_image_dims
    }

    /*pub fn clear(&self, queue: Arc<Queue>) -> Result<Box<dyn GpuFuture>, String>
    {
        let value: ClearColorValue = match self._pix_fmt.components() {

        };
        execute_in_new_primary_command_buffer(queue.clone(), None, |cbb| {cbb.clear(self, value).unwrap();})
    }*/

    pub fn clear_color(
        &self,
        command_buffer_father: &CommandBufferFather,
        value: ClearColorValue,
    ) -> Result<Box<dyn GpuFuture>, String> {
        Ok(command_buffer_father
            .execute_in_new_primary(None, |cbb| {
                cbb.clear_color(self, value).unwrap();
            })?
            .1)
    }

    pub fn clear_depth_stencil(
        &self,
        command_buffer_father: &CommandBufferFather,
        value: ClearDepthStencilValue,
    ) -> Result<Box<dyn GpuFuture>, String> {
        Ok(command_buffer_father
            .execute_in_new_primary(None, |cbb| {
                cbb.clear_depth_stencil(self, value).unwrap();
            })?
            .1)
    }

    pub fn copy_from(
        &self,
        command_buffer_father: &CommandBufferFather,
        texture: &Texture,
    ) -> Result<Box<dyn GpuFuture>, String> {
        Ok(command_buffer_father
            .execute_in_new_primary(None, |cbb| {
                cbb.copy_texture(texture, self).unwrap();
            })?
            .1)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TextureUseCase {
    General,
    Attachment,
    ReadOnly,
    Storage,
}

/// Общий Trait для заголовков всех сжатых форматов
pub trait CompressedFormat {
    fn pixel_format(&self) -> TexturePixelFormat; // Формат блока
    fn dimensions(&self) -> (u32, u32); // Размер
    fn mip_levels(&self) -> u32; // Количество mip-уровней
    fn header_size(&self) -> usize; // Размер заголовка в байтах
    fn block_size(&self) -> usize; // Размер блока в байтах
}

impl CompressedFormat for DDSHeader {
    fn pixel_format(&self) -> TexturePixelFormat {
        self.get_pixel_format()
    }
    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    fn mip_levels(&self) -> u32 {
        self.mip_map_count
    }
    fn header_size(&self) -> usize {
        128
    }
    fn block_size(&self) -> usize {
        let size = self.pixel_format().block_size();
        size as _
    }
}

impl CompressedFormat for KTXHeader {
    fn pixel_format(&self) -> TexturePixelFormat {
        self.get_pixel_format()
    }
    fn dimensions(&self) -> (u32, u32) {
        (self.pixel_width(), self.pixel_height())
    }
    fn mip_levels(&self) -> u32 {
        self.mipmap_levels()
    }
    fn header_size(&self) -> usize {
        64 + 4
    }
    fn block_size(&self) -> usize {
        let size = self.pixel_format().block_size();
        size as _
    }
}

pub(crate) trait TextureCommandSet {
    fn update_data(
        &mut self,
        allocator: Arc<StandardMemoryAllocator>,
        texture: &Texture,
        data: &[u8],
        lod_level: u32,
        array_layer: u32,
    ) -> Result<&mut Self, String>;
    fn copy_texture(&mut self, from: &Texture, to: &Texture) -> Result<&mut Self, String>;
    fn clear_color(
        &mut self,
        texture: &Texture,
        value: ClearColorValue,
    ) -> Result<&mut Self, String>;
    fn clear_depth_stencil(
        &mut self,
        texture: &Texture,
        value: ClearDepthStencilValue,
    ) -> Result<&mut Self, String>;
}

impl<Cbbt> TextureCommandSet for AutoCommandBufferBuilder<Cbbt> {
    fn update_data(
        &mut self,
        allocator: Arc<StandardMemoryAllocator>,
        texture: &Texture,
        data: &[u8],
        lod_level: u32,
        array_layer: u32,
    ) -> Result<&mut Self, String> {
        let upload_buffer = Buffer::new_slice(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            data.len() as DeviceSize,
        )
        .unwrap();
        upload_buffer.write().unwrap().copy_from_slice(data);
        let copy_info = CopyBufferToImageInfo {
            regions: [BufferImageCopy {
                image_subresource: ImageSubresourceLayers {
                    mip_level: lod_level,
                    array_layers: array_layer..(array_layer + 1),
                    aspects: {
                        let aspects = texture._pix_fmt.aspects();
                        if aspects.intersects(ImageAspects::PLANE_0) {
                            ImageAspects::PLANE_0
                        } else {
                            aspects
                        }
                    },
                },
                image_extent: [
                    1.max(texture._vk_image_dims[0] >> lod_level),
                    1.max(texture._vk_image_dims[1] >> lod_level),
                    1.max(texture._vk_image_dims[2] >> lod_level),
                ],
                image_offset: [0; 3],
                ..Default::default()
            }]
            .into(),
            ..CopyBufferToImageInfo::buffer_image(upload_buffer, texture._vk_image_access.clone())
        };
        match self.copy_buffer_to_image(copy_info.clone()) {
            Ok(s) => Ok(s),
            Err(e) => Err(format!(
                "{e}. Mip level {:?}, {}x{}",
                copy_info.regions[0].image_subresource.mip_level,
                copy_info.regions[0].image_extent[0],
                copy_info.regions[0].image_extent[1],
            )),
        }
    }

    fn clear_depth_stencil(
        &mut self,
        texture: &Texture,
        value: ClearDepthStencilValue,
    ) -> Result<&mut Self, String> {
        let subresource_range = texture.image_view().subresource_range();
        let mut ccii = ClearDepthStencilImageInfo::image(texture._vk_image_access.clone());
        ccii.clear_value = value;
        ccii.regions = [subresource_range.clone()].into();
        self.clear_depth_stencil_image(ccii).unwrap();
        Ok(self)
    }

    fn clear_color(
        &mut self,
        texture: &Texture,
        value: ClearColorValue,
    ) -> Result<&mut Self, String> {
        let mut ccii = ClearColorImageInfo::image(texture._vk_image_access.clone());
        let subresource_range = texture.image_view().subresource_range();
        ccii.clear_value = value;
        ccii.regions = [subresource_range.clone()].into();
        self.clear_color_image(ccii).unwrap();
        Ok(self)
    }

    fn copy_texture(&mut self, src: &Texture, dst: &Texture) -> Result<&mut Self, String> {
        let blit_info = BlitImageInfo::images(
            src._vk_image_view.image().clone(),
            dst._vk_image_view.image().clone(),
        );
        let src_subresource = src._vk_image_view.subresource_range();
        let src_dims = src._vk_image_access.extent();
        let dst_subresource = dst._vk_image_view.subresource_range();
        let dst_dims = dst._vk_image_access.extent();
        let blit_result = self.blit_image(BlitImageInfo {
            regions: [ImageBlit {
                src_subresource: ImageSubresourceLayers {
                    aspects: src_subresource.aspects,
                    array_layers: src_subresource.array_layers.clone(),
                    mip_level: src_subresource.mip_levels.start,
                },
                src_offsets: [[0; 3], src_dims],
                dst_subresource: ImageSubresourceLayers {
                    aspects: dst_subresource.aspects,
                    array_layers: dst_subresource.array_layers.clone(),
                    mip_level: dst_subresource.mip_levels.start,
                },
                dst_offsets: [[0; 3], dst_dims],
                ..ImageBlit::default()
            }]
            .into(),
            ..blit_info
        });
        match blit_result {
            Ok(acbb) => Ok(acbb),
            Err(err) => Err(format!("{:?}", err)),
        }
    }
}

/*update_sampler(&mut self)
{
    self._vk_sampler = TextureSampler::new(self._vk_device.clone(),
        SamplerCreateInfo {
            address_mode : [self.u_repeat, self.v_repeat, self.w_repeat],
            mag_filter : self.mag_filter,
            min_filter : self.min_filter,
            mipmap_mode : self.mip_mode,
            mip_lod_bias : self.mip_lod_bias,
            anisotropy : self.max_anisotropy,
            lod : self.min_lod..=self.max_lod,
            compare : self.compare_op,
            ..Default::default()
        }
    ).unwrap();
}*/
