mod ktx_header;
mod dds_header;
mod pixel_format;
mod types;

pub use crate::references::*;
pub use crate::shader::ShaderStructUniform;
pub type TextureRef = RcBox<Texture>;

use std::io::{Read, Seek, BufRead};
use image::io::Reader as ImageReader;

use vulkano::format::Format;

#[allow(dead_code)]
use vulkano::device::{Queue};

#[allow(dead_code)]
use vulkano::image::{
    ImmutableImage,
    AttachmentImage,
    MipmapsCount,
    view::{
        ImageView,
        ImageViewAbstract
    }
};
pub use vulkano::image::ImageDimensions as TextureDimensions;
pub use vulkano::sampler::{
    Sampler as TextureSampler,
    Filter as TextureFilter,
    MipmapMode,
    SamplerAddressMode as TextureRepeatMode,
};
use vulkano::sync::GpuFuture;
use std::sync::Arc;
//use vulkano::image::ImageFormat;

use dds_header::DDSHeader;
use ktx_header::KTXHeader;
pub use pixel_format::TexturePixelFormat;

#[allow(dead_code)]
pub enum TextureViewType
{
    Storage,
    Immutable,
    Attachment
}

#[allow(dead_code)]
pub struct Texture
{
    name: String,

    _vk_image_dims: TextureDimensions,
    _vk_image_view: Arc<dyn ImageViewAbstract + 'static>,
    _vk_sampler: Arc<TextureSampler>,
    _vk_queue: Arc<Queue>,

    min_filter: TextureFilter,
    mag_filter: TextureFilter,
    mip_mode: MipmapMode,
    u_repeat: TextureRepeatMode,
    v_repeat: TextureRepeatMode,
    w_repeat: TextureRepeatMode,
    mip_lod_bias: f32,
    max_anisotropy: f32,
    min_lod: f32,
    max_lod: f32,
}

impl ShaderStructUniform for TextureRef
{
    fn glsl_type_name() -> String
    {
        "sampler".to_string()
    }

    fn structure() -> String
    {
        String::new()
    }

    fn texture(&self) -> Option<&TextureRef>
    {
        Some(self)
    }
}

use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer, PrimaryAutoCommandBuffer, CommandBufferExecFuture};
use vulkano::sync::NowFuture;
use vulkano::image::{sys::{ImageCreationError, UnsafeImage}, ImageAccess, ImageUsage, ImageCreateFlags, immutable::SubImage, ImageLayout};
use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::memory::pool::{PotentialDedicatedAllocation, StdMemoryPoolAlloc};
use std::sync::atomic::AtomicBool;

pub struct ImmutableImagePatch<A = PotentialDedicatedAllocation<StdMemoryPoolAlloc>> {
    _image: UnsafeImage,
    _dimensions: TextureDimensions,
    _memory: A,
    _format: Format,
    initialized: AtomicBool,
    _layout: ImageLayout,
}

#[allow(dead_code)]
impl Texture
{
    pub fn builder() -> TextureBuilder
    {
        TextureBuilder {
            name: "".to_string(),
            _vk_image_dims: None,
            min_filter: TextureFilter::Nearest,
            mag_filter: TextureFilter::Nearest,
            mip_mode: MipmapMode::Nearest,
            u_repeat: TextureRepeatMode::MirroredRepeat,
            v_repeat: TextureRepeatMode::MirroredRepeat,
            w_repeat: TextureRepeatMode::MirroredRepeat,
            mip_lod_bias: 0.0,
            max_anisotropy: 1.0,
            min_lod: 0.0,
            max_lod: 0.0
        }
    }

    pub fn new_empty_1d(name: &str, length: u16, pix_fmt: TexturePixelFormat, queue: Arc<Queue>) -> Result<TextureRef, String>
    {
        let dims = TextureDimensions::Dim1d{width: length as u32, array_layers: 1};
        let mut texture_builder = Texture::builder();
        texture_builder
            .name(name)
            .build_attachment(queue, pix_fmt, dims)
    }

    pub fn new_empty_2d(name: &str, width: u16, height: u16, pix_fmt: TexturePixelFormat, queue: Arc<Queue>) -> Result<TextureRef, String>
    {
        let dims = TextureDimensions::Dim2d{width: width as u32, height: height as u32, array_layers: 1};
        let mut texture_builder = Texture::builder();
        texture_builder
            .name(name)
            .build_attachment(queue, pix_fmt, dims)
    }

    pub fn new_empty_3d(name: &str, width: u16, height: u16, layers: u16, pix_fmt: TexturePixelFormat, queue: Arc<Queue>) -> Result<TextureRef, String>
    {
        let dims = TextureDimensions::Dim3d{width: width as u32, height: height as u32, depth: layers as u32};
        let mut texture_builder = Texture::builder();
        texture_builder
            .name(name)
            .build_attachment(queue, pix_fmt, dims)
    }

    pub fn from_vk_image_view(img: Arc<dyn ImageViewAbstract>, queue: Arc<Queue>) -> Result<TextureRef, String>
    {
        let img_dims = img.image().dimensions();
        let sampler = TextureSampler::simple_repeat_linear_no_mipmap(queue.device().clone());
        println!("from_vk_image_view: {}x{}", img_dims.width(), img_dims.height());
        //TextureDimensions{width: img_dims[0], }
        Ok(RcBox::construct(Self{
            name : "".to_string(),

            _vk_image_dims : img_dims,
            _vk_image_view : img,
            _vk_sampler : sampler,
            _vk_queue : queue.clone(),

            min_filter : TextureFilter::Linear,
            mag_filter : TextureFilter::Linear,
            mip_mode : MipmapMode::Nearest,
            u_repeat : TextureRepeatMode::Repeat,
            v_repeat : TextureRepeatMode::Repeat,
            w_repeat : TextureRepeatMode::Repeat,
            mip_lod_bias : 0.0,
            max_anisotropy : 1.0,
            min_lod : 0.0,
            max_lod : 1.0,
        }))
    }

    pub fn from_file<P : AsRef<std::path::Path> + ToString>(queue: Arc<Queue>, path: P) -> Result<TextureRef, String>
    {
        let extension = path.as_ref().extension();
        let mut texture_builder = Texture::builder();
        texture_builder.name(path.to_string().as_str());

        match extension {
            None => Err(String::from("Неизвестный формат изображения")),
            Some(os_str) => {
                let reader = std::fs::File::open(path.as_ref());
                if reader.is_err() {
                    return Err(format!("Файл {} не найден.", path.to_string()));
                }
                let reader = reader.unwrap();
                let mut buf_reader = std::io::BufReader::new(reader);
                match os_str.to_str() {
                    Some("dds") | Some("ktx") => {
                        texture_builder.build_immutable_compressed(&mut buf_reader, queue)
                    },
                    _ => {
                        texture_builder.build_immutable(&mut buf_reader, queue)
                    }
                }
            }
        }
    }

    pub fn image_view(&self) -> &Arc<dyn ImageViewAbstract>
    {
        &self._vk_image_view
    }

    pub fn sampler(&self) -> &Arc<TextureSampler>
    {
        &self._vk_sampler
    }

    pub fn set_horizontal_address(&mut self, repeat_mode: TextureRepeatMode)
    {
        self.u_repeat = repeat_mode;
    }
    pub fn set_vertical_address(&mut self, repeat_mode: TextureRepeatMode)
    {
        self.v_repeat = repeat_mode;
    }
    pub fn set_depth_address(&mut self, repeat_mode: TextureRepeatMode)
    {
        self.w_repeat = repeat_mode;
    }
    pub fn set_mipmap(&mut self, mipmap_mode: MipmapMode)
    {
        self.mip_mode = mipmap_mode;
    }
    pub fn set_anisotropy(&mut self, max_aniso: f32)
    {
        self.max_anisotropy = max_aniso;
    }
    pub fn set_min_filter(&mut self, filter: TextureFilter)
    {
        self.min_filter = filter;
    }
    pub fn set_mag_filter(&mut self, filter: TextureFilter)
    {
        self.mag_filter = filter;
    }

    pub fn horizontal_address(&self) -> TextureRepeatMode
    {
        self.u_repeat
    }
    pub fn vertical_address(&self) -> TextureRepeatMode
    {
        self.v_repeat
    }
    pub fn depth_address(&self) -> TextureRepeatMode
    {
        self.w_repeat
    }
    pub fn mipmap(&self) -> MipmapMode
    {
        self.mip_mode
    }
    pub fn anisotropy(&self) -> f32
    {
        self.max_anisotropy
    }
    pub fn min_filter(&self) -> TextureFilter
    {
        self.min_filter
    }
    pub fn mag_filter(&self) -> TextureFilter
    {
        self.mag_filter
    }
    pub fn width(&self) -> u32
    {
        match self._vk_image_dims {
            TextureDimensions::Dim1d{width, ..} => width,
            TextureDimensions::Dim2d{width, ..} => width,
            TextureDimensions::Dim3d{width, ..} => width,
        }
    }
    pub fn height(&self) -> u32
    {
        match self._vk_image_dims {
            TextureDimensions::Dim1d{..} => 1,
            TextureDimensions::Dim2d{width:_, height, ..} => height,
            TextureDimensions::Dim3d{width:_, height, ..} => height,
        }
    }
    pub fn depth(&self) -> u32
    {
        match self._vk_image_dims {
            TextureDimensions::Dim1d{..} => 1,
            TextureDimensions::Dim2d{..} => 1,
            TextureDimensions::Dim3d{width:_, height:_, depth} => depth,
        }
    }
    pub fn update_sampler(&mut self)
    {
        self._vk_sampler = TextureSampler::new(
            self._vk_queue.device().clone(),
            self.mag_filter,
            self.min_filter,
            self.mip_mode,
            self.u_repeat,
            self.v_repeat,
            self.w_repeat,
            self.mip_lod_bias,
            self.max_anisotropy,
            self.min_lod,
            self.max_lod,
        ).unwrap();
    }
}

pub struct TextureBuilder
{
    _vk_image_dims: Option<TextureDimensions>,
    name: String,
    min_filter: TextureFilter,
    mag_filter: TextureFilter,
    mip_mode: MipmapMode,
    u_repeat: TextureRepeatMode,
    v_repeat: TextureRepeatMode,
    w_repeat: TextureRepeatMode,
    mip_lod_bias: f32,
    max_anisotropy: f32,
    min_lod: f32,
    max_lod: f32,
}

#[allow(dead_code)]
impl TextureBuilder
{
    pub fn name(&mut self, name: &str) -> &mut Self
    {
        self.name = name.to_string();
        self
    }
    pub fn horizontal_address(&mut self, repeat_mode: TextureRepeatMode) -> &mut Self
    {
        self.u_repeat = repeat_mode;
        self
    }
    pub fn vertical_address(&mut self, repeat_mode: TextureRepeatMode) -> &mut Self
    {
        self.v_repeat = repeat_mode;
        self
    }
    pub fn depth_address(&mut self, repeat_mode: TextureRepeatMode) -> &mut Self
    {
        self.w_repeat = repeat_mode;
        self
    }
    pub fn mipmap(&mut self, mipmap_mode: MipmapMode) -> &mut Self
    {
        self.mip_mode = mipmap_mode;
        self
    }
    pub fn anisotropy(&mut self, max_aniso: f32) -> &mut Self
    {
        self.max_anisotropy = max_aniso;
        self
    }
    pub fn build_storage(&mut self, texture_dimensions: TextureDimensions) -> &mut Self
    {
        self._vk_image_dims = Some(texture_dimensions);
        self
    }
    pub fn min_filter(&mut self, filter: TextureFilter) -> &mut Self
    {
        self.min_filter = filter;
        self
    }
    pub fn mag_filter(&mut self, filter: TextureFilter) -> &mut Self
    {
        self.mag_filter = filter;
        self
    }
    pub fn build_attachment(&mut self, queue: Arc<Queue>, pix_fmt: TexturePixelFormat, dimensions: TextureDimensions) -> Result<TextureRef, String>
    {
        let texture = AttachmentImage::sampled_input_attachment(
            queue.device().clone(),
            [dimensions.width(), dimensions.height()],
            pix_fmt.vk_format()
        ).unwrap();
        
        Ok(RcBox::construct(Texture {
            name: "".to_string(),
            _vk_image_dims: dimensions,
            _vk_image_view: ImageView::new(texture).unwrap(),
            _vk_queue: queue.clone(),
            _vk_sampler: TextureSampler::new(
                queue.device().clone(),
                self.mag_filter,
                self.min_filter,
                self.mip_mode,
                self.u_repeat,
                self.v_repeat,
                self.w_repeat,
                self.mip_lod_bias,
                self.max_anisotropy,
                self.min_lod,
                self.max_lod,
            ).unwrap(),
            mag_filter : self.mag_filter,
            min_filter : self.min_filter,
            mip_mode : self.mip_mode,
            u_repeat : self.u_repeat,
            v_repeat : self.v_repeat,
            w_repeat : self.w_repeat,
            mip_lod_bias : self.mip_lod_bias,
            max_anisotropy : self.max_anisotropy,
            min_lod : self.min_lod,
            max_lod : self.max_lod,
        }))
    }

    pub fn build_immutable_compressed<Rdr : Read + Seek + BufRead>(&mut self, reader: &mut Rdr, queue: Arc<Queue>) -> Result<TextureRef, String>
    {
        let header : Box<dyn CompressedFormat>;
        let mut header_bytes = vec!(0u8; 128);
        reader.read(header_bytes.as_mut_slice()).unwrap();
        match DDSHeader::from_bytes(header_bytes.as_slice())
        {
            Ok(hdr) => header = Box::new(hdr),
            Err(dds_error) => 

        match KTXHeader::from_bytes(header_bytes.as_slice()) {
            Ok(hdr) => header = Box::new(hdr),
            Err(ktx_error) => return Err(format!("{}; {}", dds_error, ktx_error))

        }};
        reader.seek(std::io::SeekFrom::Start(header.header_size() as u64)).unwrap();

        let mut width = header.dimensions().0;
        let mut height = header.dimensions().1;
        let mut data_size = 0;
        let dimensions = TextureDimensions::Dim2d { width: width, height: height, array_layers: 1 };

        for _i in 0..header.mip_levels() {
            let block_size = header.block_size();
            let blocks = ((width+3)/4)*((height+3)/4);
            let size = blocks as usize * block_size;
            data_size += size;
            width /= 2;
            height /= 2;
        }
        let mut compressed_img_bytes = vec!(0u8; data_size);
        reader.read(compressed_img_bytes.as_mut_slice()).unwrap();
        let (texture, tex_future) = Self::custom_immutable_image(
            compressed_img_bytes.as_slice(),
            dimensions,
            header.mip_levels(),
            header.pixel_format(),
            queue.clone()
        ).unwrap();
        
        self.max_lod = header.mip_levels() as f32;
        if self.max_lod <= 1.0 {
            self.mip_mode = MipmapMode::Nearest;
        }

        tex_future.flush().unwrap();
        Ok(RcBox::construct(Texture {
            name: "".to_string(),
            _vk_image_dims: dimensions,
            _vk_image_view: ImageView::new(texture).unwrap(),
            _vk_queue: queue.clone(),
            _vk_sampler: TextureSampler::new(
                queue.device().clone(),
                self.mag_filter,
                self.min_filter,
                self.mip_mode,
                self.u_repeat,
                self.v_repeat,
                self.w_repeat,
                self.mip_lod_bias,
                self.max_anisotropy,
                self.min_lod,
                self.max_lod,
            ).unwrap(),
            mag_filter : self.mag_filter,
            min_filter : self.min_filter,
            mip_mode : self.mip_mode,
            u_repeat : self.u_repeat,
            v_repeat : self.v_repeat,
            w_repeat : self.w_repeat,
            mip_lod_bias : self.mip_lod_bias,
            max_anisotropy : self.max_anisotropy,
            min_lod : self.min_lod,
            max_lod : self.max_lod,
        }))
    }

    pub fn build_immutable<Rdr : Read + Seek + BufRead>(&mut self, reader: &mut Rdr, queue: Arc<Queue>) -> Result<TextureRef, String>
    {
        let img_rdr = ImageReader::new(reader).with_guessed_format();
        if img_rdr.is_err() {
            return Err(String::from("Неизвестный формат изображения"));
        }
        let img_rdr = img_rdr.unwrap();
        let image = img_rdr.decode().unwrap().to_rgba8();

        let (width, height) = image.dimensions();
        let dimensions = TextureDimensions::Dim2d { width: width, height: height, array_layers: 1 };
        let image_data = image.into_raw().clone();
        let (texture, tex_future) = ImmutableImage::from_iter(
            image_data.iter().cloned(),
            dimensions,
            match self.mip_mode {
                MipmapMode::Linear  => MipmapsCount::Log2,
                MipmapMode::Nearest => MipmapsCount::One
            },
            Format::R8G8B8A8_SRGB,
            queue.clone()
        ).unwrap();

        self.max_lod = match self.mip_mode {
            MipmapMode::Linear => (width.max(height) as f32).log(2.0),
            MipmapMode::Nearest => 0.0
        };

        tex_future.flush().unwrap();
        Ok(RcBox::construct(Texture {
            name: "".to_string(),
            _vk_image_dims: dimensions,
            _vk_image_view: ImageView::new(texture).unwrap(),
            _vk_queue: queue.clone(),
            _vk_sampler: TextureSampler::new(
                queue.device().clone(),
                self.mag_filter,
                self.min_filter,
                self.mip_mode,
                self.u_repeat,
                self.v_repeat,
                self.w_repeat,
                self.mip_lod_bias,
                self.max_anisotropy,
                self.min_lod,
                self.max_lod,
            ).unwrap(),
            mag_filter : self.mag_filter,
            min_filter : self.min_filter,
            mip_mode : self.mip_mode,
            u_repeat : self.u_repeat,
            v_repeat : self.v_repeat,
            w_repeat : self.w_repeat,
            mip_lod_bias : self.mip_lod_bias,
            max_anisotropy : self.max_anisotropy,
            min_lod : self.min_lod,
            max_lod : self.max_lod,
        }))
    }

    pub fn custom_immutable_image(
        data: &[u8],
        dimensions: TextureDimensions,
        mip_map_count: u32,
        format: TexturePixelFormat,
        queue: Arc<Queue>,
    ) -> Result<
        (
            Arc<ImmutableImage>,
            CommandBufferExecFuture<NowFuture, PrimaryAutoCommandBuffer>,
        ),
        ImageCreationError,
    >
    {
        
        let usage = ImageUsage {
            transfer_destination: true,
            transfer_source: true,
            sampled: true,
            ..ImageUsage::none()
        };
        let flags = ImageCreateFlags::none();
        let layout = ImageLayout::ShaderReadOnlyOptimal;

        let (mut image, initializer) = ImmutableImage::uninitialized(
            queue.device().clone(),
            dimensions,
            format.vk_format(),
            if mip_map_count > 1 { MipmapsCount::Log2 } else { MipmapsCount::One },
            usage,
            flags,
            layout,
            queue.device().active_queue_families(),
        )?;

        let init = SubImage::new(initializer, 0, 1, 0, 1, ImageLayout::ShaderReadOnlyOptimal);

        let mut cbb = AutoCommandBufferBuilder::primary(
            queue.device().clone(),
            queue.family(),
            CommandBufferUsage::MultipleSubmit,
        )?;
        
        let mut width = dimensions.width();
        let mut height = dimensions.height();
        let mut data_offset = 0usize;
        
        let u64s = match format {
            TexturePixelFormat::Etc2RGB   => 1,
            TexturePixelFormat::Etc2RGBA  => 2,
            TexturePixelFormat::EacR11    => 1,
            TexturePixelFormat::EacRG11   => 2,
            TexturePixelFormat::S3tcDXT1a => 1,
            TexturePixelFormat::S3tcDXT1  => 1,
            TexturePixelFormat::S3tcDXT3  => 2,
            TexturePixelFormat::S3tcDXT5  => 2,
            _ => {
                return Err(ImageCreationError::FormatNotSupported);
            }
        };
        //println!("Попытка загрузить файл формата {}", format.stringify());
        for i in 0..mip_map_count {
            //println!("mip-map {}/{}", i, mip_map_count);
            let block_size = u64s * 8;
            let blocks = ((width+3)/4)*((height+3)/4);
            let size = (blocks * block_size) as usize;

            let source = CpuAccessibleBuffer::from_iter(
                queue.device().clone(),
                BufferUsage::transfer_source(),
                false,
                data[data_offset..data_offset+size].iter().cloned(),
            ).unwrap();
            data_offset += size;
            cbb.copy_buffer_to_image_dimensions(
                source,
                init.clone(),
                [0, 0, 0],
                [width, height, 1],
                0,
                dimensions.array_layers(),
                i,
            )
            .unwrap();
            width  /= 2;
            height /= 2;
        }

        let cb = cbb.build().unwrap();

        let future = match cb.execute(queue) {
            Ok(f) => f,
            Err(e) => unreachable!("{:?}", e),
        };
        
        unsafe {
            (&mut image as *mut Arc<ImmutableImage> as *mut Arc<ImmutableImagePatch>).as_ref().unwrap().initialized.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        Ok((image, future))
    }
}

pub trait CompressedFormat
{
    fn pixel_format(&self) -> TexturePixelFormat;
    fn dimensions(&self) -> (u32, u32);
    fn mip_levels(&self) -> u32;
    fn header_size(&self) -> usize;
    fn block_size(&self) -> usize;
}

impl CompressedFormat for DDSHeader {
    fn pixel_format(&self) -> TexturePixelFormat
    {
        self.get_pixel_format()
    }
    fn dimensions(&self) -> (u32, u32)
    {
        (self.width, self.height)
    }
    fn mip_levels(&self) -> u32
    {
        self.mip_map_count
    }
    fn header_size(&self) -> usize
    {
        128
    }
    fn block_size(&self) -> usize
    {
        let u64s = match self.pixel_format()
        {
            TexturePixelFormat::Etc2RGB   => 1,
            TexturePixelFormat::Etc2RGBA  => 2,
            TexturePixelFormat::EacR11    => 1,
            TexturePixelFormat::EacRG11   => 2,
            TexturePixelFormat::S3tcDXT1a => 1,
            TexturePixelFormat::S3tcDXT1  => 1,
            TexturePixelFormat::S3tcDXT3  => 2,
            TexturePixelFormat::S3tcDXT5  => 2,
            _ => 0
        };
        u64s * 8
    }
}

impl CompressedFormat for KTXHeader {
    fn pixel_format(&self) -> TexturePixelFormat
    {
        self.get_pixel_format()
    }
    fn dimensions(&self) -> (u32, u32)
    {
        (self.pixel_width(), self.pixel_height())
    }
    fn mip_levels(&self) -> u32
    {
        self.mipmap_levels()
    }
    fn header_size(&self) -> usize
    {
        64 + 4
    }
    fn block_size(&self) -> usize
    {
        let u64s = match self.pixel_format()
        {
            TexturePixelFormat::Etc2RGB   => 1,
            TexturePixelFormat::Etc2RGBA  => 2,
            TexturePixelFormat::EacR11    => 1,
            TexturePixelFormat::EacRG11   => 2,
            TexturePixelFormat::S3tcDXT1a => 1,
            TexturePixelFormat::S3tcDXT1  => 1,
            TexturePixelFormat::S3tcDXT3  => 2,
            TexturePixelFormat::S3tcDXT5  => 2,
            _ => 0
        };
        u64s * 8
    }
}