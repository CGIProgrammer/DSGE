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
use vulkano::device::{Queue, Device};

#[allow(dead_code)]
use vulkano::image::{
    ImmutableImage,
    AttachmentImage,
    MipmapsCount,
    view::{
        ImageView,
        ImageViewAbstract,
        //ImageViewCreateInfo
    }
};

pub use vulkano::image::ImageDimensions as TextureDimensions;
pub use vulkano::image::view::ImageViewType as TextureType;

pub use vulkano::sampler::{
    Sampler as TextureSampler,
    SamplerCreateInfo,
    Filter as TextureFilter,
    SamplerMipmapMode as MipmapMode,
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

/// Текстура (она же изображение)
#[allow(dead_code)]
pub struct Texture
{
    name: String,

    _vk_image_dims: TextureDimensions,
    _vk_image_view: Arc<dyn ImageViewAbstract + 'static>,
    _vk_sampler: Arc<TextureSampler>,
    _vk_device: Arc<Device>,

    _pix_fmt: TexturePixelFormat,

    min_filter: TextureFilter,
    mag_filter: TextureFilter,
    mip_mode: MipmapMode,
    u_repeat: TextureRepeatMode,
    v_repeat: TextureRepeatMode,
    w_repeat: TextureRepeatMode,
    mip_lod_bias: f32,
    max_anisotropy: Option<f32>,
    min_lod: f32,
    max_lod: f32,
}
/*
impl std::hash::Hash for Texture
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self._vk_image_view.hash(state);
    }
}
*/
impl ShaderStructUniform for TextureRef
{
    fn glsl_type_name() -> String
    {
        String::new()
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

/// Патч-структура, аналогичная ImmutableImage из vulkano.
/// Предназначена для изменения приватного флага initialized.
/// Эта структура в составе vulkano не позволяет только генерировать mip-уровни но не загружать их.
/// В vulkano имеются функции, которые могут это делать,
/// однако они требуют установки вышеупомянутого приватного флага.
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
            max_anisotropy: None,
            min_lod: 0.0,
            max_lod: 0.0
        }
    }

    pub fn name(&self) -> &String
    {
        &self.name
    }

    pub fn ty(&self) -> TextureType
    {
        self._vk_image_view.view_type()
    }

    /// Создаёт пустое изображение, которое можно использовать для записи
    pub fn new_empty(name: &str, dims: TextureDimensions, pix_fmt: TexturePixelFormat, device: Arc<Device>) -> Result<Texture, String>
    {
        let mut texture_builder = Texture::builder();
        texture_builder.name(name);
        texture_builder.build_mutable(device, pix_fmt, dims)
    }

    /// Тоже что и new_empty, только Texture оборачивается в мьютекс
    pub fn new_empty_mutex(name: &str, dims: TextureDimensions, pix_fmt: TexturePixelFormat, device: Arc<Device>) -> Result<TextureRef, String>
    {
        Ok(RcBox::construct(Self::new_empty(name, dims, pix_fmt, device)?))
    }

    /// Получает формат пикселя текстуры
    pub fn pix_fmt(&self) -> TexturePixelFormat
    {
        self._pix_fmt
    }

    /// Создаёт `TextureRef` на основе `ImageViewAbstract`.
    /// В основном используется для представления swapchain изображения в виде текстуры
    /// для вывода результата рендеринга
    pub fn from_vk_image_view(img: Arc<dyn ImageViewAbstract>, device: Arc<Device>) -> Result<TextureRef, String>
    {
        let img_dims = img.image().dimensions();
        let sampler = TextureSampler::new(device.clone(), SamplerCreateInfo {
            address_mode : [TextureRepeatMode::ClampToEdge, TextureRepeatMode::ClampToEdge, TextureRepeatMode::ClampToEdge],
            mipmap_mode : MipmapMode::Nearest,
            anisotropy : None,
            min_filter : TextureFilter::Nearest,
            mag_filter : TextureFilter::Nearest,
            ..Default::default()
        }).unwrap();

        let pix_fmt = TexturePixelFormat::from_vk_format(img.image().format())?;
        println!("from_vk_image_view: {}x{}", img_dims.width(), img_dims.height());
        //TextureDimensions{width: img_dims[0], }
        Ok(RcBox::construct(Self{
            name : "".to_string(),

            _vk_image_dims : img_dims.into(),
            _vk_image_view : img,
            _vk_sampler : sampler,
            _vk_device : device.clone(),

            _pix_fmt : pix_fmt,

            min_filter : TextureFilter::Nearest,
            mag_filter : TextureFilter::Nearest,
            mip_mode : MipmapMode::Nearest,
            u_repeat : TextureRepeatMode::Repeat,
            v_repeat : TextureRepeatMode::Repeat,
            w_repeat : TextureRepeatMode::Repeat,
            mip_lod_bias : 0.0,
            max_anisotropy : None,
            min_lod : 0.0,
            max_lod : 1.0,
        }))
    }

    pub fn from_file_mutex<P : AsRef<std::path::Path> + ToString>(queue: Arc<Queue>, path: P) -> Result<TextureRef, String>
    {
        Ok(RcBox::construct(Self::from_file(queue, path)?))
    }

    /// Загрузка изображения-текстуры из файла.
    /// Поддерживаются форматы dds, ktx и все форматы, поддерживаемые crate'ом image
    pub fn from_file<P : AsRef<std::path::Path> + ToString>(queue: Arc<Queue>, path: P) -> Result<Texture, String>
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
                        //panic!("Загрузка текстур форматов кроме dds и ktx отключена");
                        texture_builder.build_immutable(&mut buf_reader, queue)
                    }
                }
            }
        }
    }

    /// Возвращает представление изображения
    pub fn image_view(&self) -> &Arc<dyn ImageViewAbstract>
    {
        &self._vk_image_view
    }

    /// Возвращает сэмплер изображения
    pub fn sampler(&self) -> &Arc<TextureSampler>
    {
        &self._vk_sampler
    }

    /// Задаёт адресный режим доступа к изображению по горизонтали.
    pub fn set_horizontal_address(&mut self, repeat_mode: TextureRepeatMode)
    {
        self.u_repeat = repeat_mode;
    }

    /// Задаёт адресный режим доступа к изображению по вертикали.
    pub fn set_vertical_address(&mut self, repeat_mode: TextureRepeatMode)
    {
        self.v_repeat = repeat_mode;
    }

    /// Задаёт адресный режим доступа к воксельному изображению по глубине.
    pub fn set_depth_address(&mut self, repeat_mode: TextureRepeatMode)
    {
        self.w_repeat = repeat_mode;
    }

    /// Задаёт режим использования mip-уровней.
    /// `Linear` - плавный переход от уровня у уровню
    /// `Nearest` - резкий переход от уровня у уровню
    pub fn set_mipmap(&mut self, mipmap_mode: MipmapMode)
    {
        self.mip_mode = mipmap_mode;
    }

    /// Задаёт степень анизотропной фльтрации.
    /// max_aniso - степень фильтрации
    pub fn set_anisotropy(&mut self, max_aniso: Option<f32>)
    {
        self.max_anisotropy = max_aniso;
    }

    /// Задаёт фильтрацию при сжатии ихображения
    pub fn set_min_filter(&mut self, filter: TextureFilter)
    {
        self.min_filter = filter;
    }

    /// Задаёт фильтрацию при растягивании изображения
    pub fn set_mag_filter(&mut self, filter: TextureFilter)
    {
        self.mag_filter = filter;
    }

    /// Набор геттеров для получения вышеуказанных полей.
    
    /// Режим адресации по горизонтали
    pub fn horizontal_address(&self) -> TextureRepeatMode
    {
        self.u_repeat
    }

    /// Режим адресации по вертикали
    pub fn vertical_address(&self) -> TextureRepeatMode
    {
        self.v_repeat
    }

    /// Режим адресации по глубине
    pub fn depth_address(&self) -> TextureRepeatMode
    {
        self.w_repeat
    }

    /// Режим mip-текстурирования
    pub fn mipmap(&self) -> MipmapMode
    {
        self.mip_mode
    }

    /// Степень анизотропии
    pub fn anisotropy(&self) -> Option<f32>
    {
        self.max_anisotropy
    }

    /// Фильтр сжатия изображения
    pub fn min_filter(&self) -> TextureFilter
    {
        self.min_filter
    }

    /// Фильтр растягивания изображения
    pub fn mag_filter(&self) -> TextureFilter
    {
        self.mag_filter
    }

    /// Ширина (длина для 1D текстур)
    pub fn width(&self) -> u32
    {
        match self._vk_image_dims {
            TextureDimensions::Dim1d{width, ..} => width,
            TextureDimensions::Dim2d{width, ..} => width,
            TextureDimensions::Dim3d{width, ..} => width,
        }
    }

    /// Высота (1 писель для 1D текстур)
    pub fn height(&self) -> u32
    {
        match self._vk_image_dims {
            TextureDimensions::Dim1d{..} => 1,
            TextureDimensions::Dim2d{width:_, height, ..} => height,
            TextureDimensions::Dim3d{width:_, height, ..} => height,
        }
    }

    /// Глубина (1 пиксель для 1D и 2D текстур)
    pub fn depth(&self) -> u32
    {
        match self._vk_image_dims {
            TextureDimensions::Dim1d{..} => 1,
            TextureDimensions::Dim2d{..} => 1,
            TextureDimensions::Dim3d{width:_, height:_, depth} => depth,
        }
    }

    pub fn dims(&self) -> TextureDimensions
    {
        self._vk_image_dims
    }

    /// Обновление сэмплера текстуры
    pub fn update_sampler(&mut self)
    {
        self._vk_sampler = TextureSampler::new(self._vk_device.clone(), SamplerCreateInfo {
            address_mode : [self.u_repeat, self.v_repeat, self.w_repeat],
            mag_filter : self.mag_filter,
            min_filter : self.min_filter,
            mipmap_mode : self.mip_mode,
            mip_lod_bias : self.mip_lod_bias,
            anisotropy : self.max_anisotropy,
            lod : self.min_lod..=self.max_lod,
            ..Default::default()
        }).unwrap();
    }

    pub fn clear_color(&mut self, queue: Arc<Queue>)
    {
        let device = queue.device().clone();
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            device.clone(),
            queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();
        match self._pix_fmt.components() {
            1 => {
                let val = if self._pix_fmt.is_depth() { 1.0 } else { 0.0 };
                command_buffer_builder.clear_color_image(self._vk_image_view.image(), val.into()).unwrap();
            },
            2 => {command_buffer_builder.clear_color_image(self._vk_image_view.image(), [0.0; 2].into()).unwrap(); },
            3 => {command_buffer_builder.clear_color_image(self._vk_image_view.image(), [0.0; 3].into()).unwrap(); },
            4 => {command_buffer_builder.clear_color_image(self._vk_image_view.image(), [0.0; 4].into()).unwrap(); },
            _ => ()
        };
        let command_buffer = command_buffer_builder.build().unwrap();
        let future = command_buffer.execute(queue).unwrap();
        drop(future);
    }

    pub fn copy_from(&self, texture: &Texture, queue: Arc<Queue>)
    {
        let device = queue.device().clone();
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            device.clone(),
            queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        command_buffer_builder.blit_image(
            texture._vk_image_view.image(),
            [0, 0, 0], [texture.width() as i32, texture.height() as i32, texture.depth() as i32],
            0, 0,
            self._vk_image_view.image(),
            [0, 0, 0], [self.width() as i32, self.height() as i32, self.depth() as i32],
            0, 0,
            self._vk_image_dims.array_layers().min(texture._vk_image_dims.array_layers()),
            TextureFilter::Nearest
        ).unwrap();
        let command_buffer = command_buffer_builder.build().unwrap();
        let future = command_buffer.execute(queue);

        match future {
            Ok(_) => (),
            Err(e) => {
                panic!("Не удалось скопировать текстуру: {:?}", e);
            }
        };
    }
}

/// Строитель для `TextureRef`
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
    max_anisotropy: Option<f32>,
    min_lod: f32,
    max_lod: f32,
}

#[allow(dead_code)]
impl TextureBuilder
{
    /// Задаёт имя
    pub fn name(&mut self, name: &str) -> &mut Self
    {
        self.name = name.to_string();
        self
    }

    /// Задаёт адресный режим по горизонтали
    pub fn horizontal_address(&mut self, repeat_mode: TextureRepeatMode) -> &mut Self
    {
        self.u_repeat = repeat_mode;
        self
    }

    /// Задаёт адресный режим по вертикали
    pub fn vertical_address(&mut self, repeat_mode: TextureRepeatMode) -> &mut Self
    {
        self.v_repeat = repeat_mode;
        self
    }

    /// Задаёт адресный режим по глубине
    pub fn depth_address(&mut self, repeat_mode: TextureRepeatMode) -> &mut Self
    {
        self.w_repeat = repeat_mode;
        self
    }

    /// Задаёт режим mip-уровней
    pub fn mipmap(&mut self, mipmap_mode: MipmapMode) -> &mut Self
    {
        self.mip_mode = mipmap_mode;
        self
    }

    /// Задаёт степень анизотропии
    pub fn anisotropy(&mut self, max_aniso: Option<f32>) -> &mut Self
    {
        self.max_anisotropy = max_aniso;
        self
    }
    
    /// Задаёт фильтр сжатия
    pub fn min_filter(&mut self, filter: TextureFilter) -> &mut Self
    {
        self.min_filter = filter;
        self
    }

    /// Задаёт фильтр растягивания
    pub fn mag_filter(&mut self, filter: TextureFilter) -> &mut Self
    {
        self.mag_filter = filter;
        self
    }

    pub fn build_mutable_mutex(self, device: Arc<Device>, pix_fmt: TexturePixelFormat, dimensions: TextureDimensions) -> Result<TextureRef, String>
    {
        Ok(RcBox::construct(self.build_mutable(device, pix_fmt, dimensions)?))
    }

    /// Строит изменяемое изображение
    pub fn build_mutable(self, device: Arc<Device>, pix_fmt: TexturePixelFormat, dimensions: TextureDimensions) -> Result<Texture, String>
    {
        let texture = AttachmentImage::with_usage(
            device.clone(),
            [dimensions.width(), dimensions.height()],
            pix_fmt.vk_format(),
            ImageUsage {
                transfer_source: true,
                transfer_destination: true,
                sampled: true,
                storage: false,
                color_attachment: true,
                depth_stencil_attachment: true,
                transient_attachment: false,
                input_attachment: false,
            }
        ).unwrap();

        let sampler = TextureSampler::new(device.clone(), SamplerCreateInfo {
            address_mode : [self.u_repeat, self.v_repeat, self.w_repeat],
            mag_filter : self.mag_filter,
            min_filter : self.min_filter,
            mipmap_mode : self.mip_mode,
            mip_lod_bias : self.mip_lod_bias,
            anisotropy : self.max_anisotropy,
            lod : self.min_lod..=self.max_lod,
            ..Default::default()
        }).unwrap();

        //img.
        Ok(Texture {
            name: self.name.to_string(),
            _vk_image_dims: dimensions,
            _vk_image_view: ImageView::new_default(texture).unwrap(),
            _vk_device: device.clone(),
            _pix_fmt: pix_fmt,
            _vk_sampler: sampler,
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
        })
    }
    
    pub fn build_immutable_compressed_mutex<Rdr : Read + Seek + BufRead>(self, reader: &mut Rdr, queue: Arc<Queue>) -> Result<TextureRef, String>
    {
        Ok(RcBox::construct(self.build_immutable_compressed(reader, queue)?))
    }

    /// Строит неизменяемое сжатое изображение
    pub fn build_immutable_compressed<Rdr : Read + Seek + BufRead>(mut self, reader: &mut Rdr, queue: Arc<Queue>) -> Result<Texture, String>
    {
        let device = queue.device();
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
        //let mip_levels = MipmapsCount::Specific(header.mip_levels());
        /*let (texture, tex_future) = ImmutableImage::from_iter(
            compressed_img_bytes,
            dimensions,
            mip_levels,
            header.pixel_format().vk_format(),
            queue.clone()
        ).unwrap();*/

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

        let sampler = TextureSampler::new(device.clone(), SamplerCreateInfo {
            address_mode : [self.u_repeat, self.v_repeat, self.w_repeat],
            mag_filter : self.mag_filter,
            min_filter : self.min_filter,
            mipmap_mode : self.mip_mode,
            mip_lod_bias : self.mip_lod_bias,
            anisotropy : self.max_anisotropy,
            lod : self.min_lod..=self.max_lod,
            ..Default::default()
        }).unwrap();

        tex_future.flush().unwrap();
        Ok(Texture {
            name: self.name.clone(),
            _vk_image_dims: dimensions,
            _vk_image_view: ImageView::new_default(texture).unwrap(),
            _vk_device: device.clone(),
            _pix_fmt: header.pixel_format(),
            _vk_sampler: sampler,
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
        })
    }
    
    pub fn build_immutable_mutex<Rdr : Read + Seek + BufRead>(self, reader: &mut Rdr, queue: Arc<Queue>) -> Result<TextureRef, String>
    {
        Ok(RcBox::construct(self.build_immutable(reader, queue)?))
    }

    /// Строит неизменяемое несжатое изображение
    pub fn build_immutable<Rdr : Read + Seek + BufRead>(mut self, reader: &mut Rdr, queue: Arc<Queue>) -> Result<Texture, String>
    {
        let pix_fmt = TexturePixelFormat::from_vk_format(Format::R8G8B8A8_SRGB)?;
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

        let sampler = TextureSampler::new(queue.device().clone(), SamplerCreateInfo {
            address_mode : [self.u_repeat, self.v_repeat, self.w_repeat],
            mag_filter : self.mag_filter,
            min_filter : self.min_filter,
            mipmap_mode : self.mip_mode,
            mip_lod_bias : self.mip_lod_bias,
            anisotropy : self.max_anisotropy,
            lod : self.min_lod..=self.max_lod,
            ..Default::default()
        }).unwrap();

        tex_future.flush().unwrap();
        Ok(Texture {
            name: self.name.clone(),
            _vk_image_dims: dimensions,
            _vk_image_view: ImageView::new_default(texture).unwrap(),
            _vk_device: queue.device().clone(),
            _pix_fmt: pix_fmt,
            _vk_sampler: sampler,
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
        })
    }

    /// Функция, аналогичная фуикции immutable_image структуры ImmutableImage из vulkano.
    fn custom_immutable_image(
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
        //println!("Dimensions = {:?}", dimensions);
        //println!("mipmap_count = {:?}", mip_map_count);
        let device = queue.device();
        let usage = ImageUsage {
            transfer_destination: true,
            transfer_source: true,
            sampled: true,
            ..ImageUsage::none()
        };
        let flags = ImageCreateFlags::none();
        let layout = ImageLayout::ShaderReadOnlyOptimal;

        let (mut image, initializer) = ImmutableImage::uninitialized(
            device.clone(),
            dimensions,
            format.vk_format(),
            if mip_map_count > 1 { MipmapsCount::Log2 } else { MipmapsCount::One },
            usage,
            flags,
            layout,
            device.active_queue_families(),
        )?;

        let init = SubImage::new(initializer, 0, 1, 0, 1, ImageLayout::ShaderReadOnlyOptimal);

        let mut cbb = AutoCommandBufferBuilder::primary(
            device.clone(),
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
        
        // Загрузка mip-уровней.
        for i in 0..mip_map_count {
            //println!("mip-map {}/{}", i, mip_map_count);
            let block_size = u64s * 8;
            let blocks = ((width+3)/4)*((height+3)/4);
            let size = (blocks * block_size) as usize;
            println!("Загрузка mip-уровня {} ({}x{})", i, width, height);
            let source = CpuAccessibleBuffer::from_iter(
                device.clone(),
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
            width  = width  / 2;
            height = height / 2;
            if width==0 || height==0 {
                break;
            }
        }

        let cb = cbb.build().unwrap();

        let future = match cb.execute(queue) {
            Ok(f) => f,
            Err(e) => unreachable!("{:?}", e),
        };
        
        // Установка флага инициализации внутреннего изображения vulkano
        unsafe {
            (&mut image as *mut Arc<ImmutableImage> as *mut Arc<ImmutableImagePatch>).as_ref().unwrap().initialized.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        Ok((image, future))
    }
}

/// Общий Trait для заголовков всех сжатых форматов
pub trait CompressedFormat
{
    fn pixel_format(&self) -> TexturePixelFormat;   // Формат блока
    fn dimensions(&self) -> (u32, u32);             // Размер
    fn mip_levels(&self) -> u32;                    // Количество mip-уровней
    fn header_size(&self) -> usize;                 // Размер заголовка в байтах
    fn block_size(&self) -> usize;                  // Размер блока в байтах
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

pub trait TextureTypeGlsl
{
    fn glsl_sampler_name(&self) -> &'static str;
}

impl TextureTypeGlsl for TextureType
{
    fn glsl_sampler_name(&self) -> &'static str
    {
        match self
        {
            TextureType::Dim1d => "sampler1D",
            TextureType::Dim1dArray => "sampler1DArray",
            TextureType::Dim2d => "sampler2D",
            TextureType::Dim2dArray => "sampler2DArray",
            TextureType::Cube => "samplerCube",
            TextureType::Dim3d => "sampler3D",
            TextureType::CubeArray => "samplerCubeArray",
        }
    }
}