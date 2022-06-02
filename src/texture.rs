mod ktx_header;
mod dds_header;
mod pixel_format;
mod types;

pub use crate::references::*;
pub use crate::shader::ShaderStructUniform;
//pub type TextureRef = RcBox<Texture>;
pub use pixel_format::TexturePixelFormatFeatures;
pub use types::*;

use std::ffi::c_void;
use std::io::{Read, Seek, BufRead};
use image::io::Reader as ImageReader;
use image::EncodableLayout;

use vulkano::format::{Format};

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
        ImageViewCreateInfo
    }
};

pub use vulkano::image::ImageDimensions as TextureDimensions;
pub use vulkano::image::view::ImageViewType as TextureView;

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
#[derive(Clone)]
pub struct Texture
{
    name: String,

    _vk_image_dims: TextureDimensions,
    _vk_image_view: Arc<dyn ImageViewAbstract + 'static>,
    _vk_image_access: Arc<dyn ImageAccess + 'static>,
    _vk_sampler: Arc<TextureSampler>,
    _vk_device: Arc<Device>,

    _pix_fmt: TexturePixelFormat,

    is_cubemap: bool,
    is_array: bool,

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
impl ShaderStructUniform for Texture
{
    fn glsl_type_name() -> String
    {
        String::new()
    }

    fn structure() -> String
    {
        String::new()
    }

    fn texture(&self) -> Option<&Texture>
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
    pub fn box_id(&self) -> u32
    {
        self._vk_sampler.as_ref() as *const TextureSampler as u32 ^
        self._vk_image_access.as_ref() as *const dyn ImageAccess as *const c_void as u32 ^
        self._vk_image_view.as_ref() as *const dyn ImageViewAbstract as *const c_void as u32
    }

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

    pub fn ty(&self) -> TextureView
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

    pub fn array_layer_as_texture(&self, layer: u32) -> Result<Texture, String>
    {
        let view_type = match self._vk_image_dims {
            TextureDimensions::Dim1d {..} => TextureView::Dim1d,
            TextureDimensions::Dim2d {..} => TextureView::Dim2d,
            TextureDimensions::Dim3d {..} => TextureView::Dim2d
        };
        Texture::from_vk_image_view(
            ImageView::new(
                self._vk_image_access.clone(),
                ImageViewCreateInfo {
                    view_type: view_type,
                    format: Some(self._pix_fmt),
                    array_layers: layer..(layer+1),
                    ..Default::default()
                }
            ).unwrap(),
            self._vk_device.clone()
        )
    }

    /*/// Тоже что и new_empty, только Texture оборачивается в мьютекс
    pub fn new_empty_mutex(name: &str, dims: TextureDimensions, pix_fmt: TexturePixelFormat, device: Arc<Device>) -> Result<TextureRef, String>
    {
        Ok(RcBox::construct(Self::new_empty(name, dims, pix_fmt, device)?))
    }*/

    /// Получает формат пикселя текстуры
    pub fn pix_fmt(&self) -> TexturePixelFormat
    {
        self._pix_fmt
    }

    pub fn copy(from: &Texture, to: &Texture, cbb: Option<&mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>, queue: Option<Arc<Queue>>)
    {
        let execute = cbb.is_none();
        let mut _cbb = None;
        
        let cbb = match cbb {
            Some(builder) => builder,
            None => {
                let queue = queue.as_ref().unwrap().clone();
                _cbb = Some(AutoCommandBufferBuilder::primary(
                    queue.device().clone(),
                    queue.family(),
                    CommandBufferUsage::OneTimeSubmit
                ).unwrap());
                _cbb.as_mut().unwrap()
            }
        };
        let (from_dims, from_array_layers) = match from._vk_image_dims {
            TextureDimensions::Dim1d { width, array_layers } => 
                ([width as _, 1, 1], array_layers),
            TextureDimensions::Dim2d { width, height, array_layers} =>
                ([width as _, height as _, 1], array_layers),
            TextureDimensions::Dim3d { width, height, depth} =>
                ([width as _, height as _, depth as _], 1),
        };
        let (to_dims, to_array_layers) = match to._vk_image_dims {
            TextureDimensions::Dim1d { width, array_layers } => 
                ([width as _, 1, 1], array_layers),
            TextureDimensions::Dim2d { width, height, array_layers} =>
                ([width as _, height as _, 1], array_layers),
            TextureDimensions::Dim3d { width, height, depth} =>
                ([width as _, height as _, depth as _], 1),
        };
        cbb.blit_image(
            from._vk_image_access.clone(), [0,0,0], from_dims,
            0, 0,
            to._vk_image_access.clone(), [0,0,0], to_dims,
            0,0, from_array_layers.min(to_array_layers),
            TextureFilter::Nearest
        ).unwrap();
        if execute {
            let cb = _cbb.unwrap().build().unwrap();
            drop(cb.execute(queue.unwrap()).unwrap());
        }
    }

    pub fn load_data<P : AsRef<std::path::Path> + ToString>(&self, queue: Arc<Queue>, path: P) -> Result<(), String>
    {
        let extension = path.as_ref().extension();
        //let mut texture_builder = Texture::builder();
        //texture_builder.name(path.to_string().as_str());
        match extension {
            None => Err(String::from("Неизвестный формат изображения")),
            Some(os_str) => {
                let reader = std::fs::File::open(path.as_ref());
                if reader.is_err() {
                    return Err(format!("Файл {} не найден.", path.to_string()));
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
                        Err("load_data не поддерживает обновление сжатых текстур".to_string())
                    },
                    _ => {
                        let mut cbb = AutoCommandBufferBuilder::primary(
                            self._vk_image_view.device().clone(),
                            queue.family(),
                            CommandBufferUsage::OneTimeSubmit
                        ).unwrap();
                        //let mut data = Vec::new();
                        //buf_reader.read_to_end(&mut data).unwrap();
                        let data = image.as_raw().clone();
                        
                        let cpuab = CpuAccessibleBuffer::from_iter(
                            self._vk_device.clone(), BufferUsage::transfer_source(), false, data
                        ).unwrap();
                        cbb.copy_buffer_to_image(cpuab, self._vk_image_access.clone()).unwrap();
                        /*self._vk_image_view = ImageView::new(self._vk_image_access.clone(), ImageViewCreateInfo{
                            format: Some(TexturePixelFormat::R8G8B8A8_SRGB),
                            view_type: self._vk_image_view.view_type(),
                            component_mapping: self._vk_image_view.component_mapping(),
                            aspects: self._vk_image_view.aspects().clone(),
                            array_layers: self._vk_image_view.array_layers(),
                            mip_levels: self._vk_image_view.mip_levels(),
                            ..Default::default()
                        }).unwrap();*/
                        drop(cbb.build().unwrap().execute(queue).unwrap());
                        Ok(())
                    }
                }
            }
        }
    }

    pub fn to_pixel_format(&self, queue: Arc<Queue>, pix_fmt: TexturePixelFormat) -> Texture
    {
        let tex = Texture::new_empty(self.name.as_str(), self._vk_image_dims, pix_fmt, self._vk_device.clone()).unwrap();
        let mut cbb = AutoCommandBufferBuilder::primary(
            self._vk_image_view.device().clone(),
            queue.family(),
            CommandBufferUsage::OneTimeSubmit
        ).unwrap();
        let (dims, array_layers) = match self._vk_image_dims {
            TextureDimensions::Dim1d { width, array_layers } => {
                ([width as i32, 1, 1], array_layers)
            },
            TextureDimensions::Dim2d { width, height, array_layers } => {
                ([width as i32, height as i32, 1], array_layers)
            },
            TextureDimensions::Dim3d { width, height, depth } => {
                ([width as i32, height as i32, depth as i32], 1)
            }
        };
        cbb.blit_image(
            self._vk_image_access.clone(),
            [0, 0, 0],
            dims,
            0,
            0,
            tex._vk_image_access.clone(),
            [0, 0, 0],
            dims,
            0,
            0,
            array_layers,
            TextureFilter::Nearest
        ).unwrap();
        let future = cbb.build().unwrap().execute(queue.clone()).unwrap();
        drop(future);
        tex
    }

    pub fn save<P : AsRef<std::path::Path> + ToString>(&self, queue: Arc<Queue>, path: P)
    {
        //if self._pix_fmt.compression().is_some() || self._pix_fmt.block_extent() != [1,1,1] {
        //    panic!("Сохранение сжатых текстур не поддерживается. Да и зачем оно вообще нужно?");
        //}
        let subpix_count = self._pix_fmt.subpixels();
        let block_size = self._pix_fmt.block_size().unwrap() as u32;
        let pix_fmt = match subpix_count {
            1 => TexturePixelFormat::R8G8B8A8_SRGB,
            2 => TexturePixelFormat::R8G8B8A8_SRGB,
            3 => TexturePixelFormat::R8G8B8A8_SRGB,
            4 => TexturePixelFormat::R8G8B8A8_SRGB,
            _ => panic!("Текстуры с {} компонентами не поддерживаются", subpix_count)
        };
        let img = self.to_pixel_format(queue.clone(), pix_fmt);
        let mut cbb = AutoCommandBufferBuilder::primary(
            self._vk_image_view.device().clone(),
            queue.family(),
            CommandBufferUsage::OneTimeSubmit
        ).unwrap();
        
        let cpuab = unsafe { CpuAccessibleBuffer::<[u8]>::uninitialized_array(
            queue.device().clone(),
            (block_size * self._vk_image_dims.num_texels()) as _,
            BufferUsage::transfer_destination(),
            false
        ).unwrap() };
        
        cbb.copy_image_to_buffer(img._vk_image_access.clone(), cpuab.clone()).unwrap();
        drop(cbb.build().unwrap().execute(queue).unwrap());
        let buf = cpuab.read().unwrap().to_vec();
        match subpix_count {
            1 => {
                let img = image::GrayImage::from_raw(img.width(), img.height(), buf).unwrap();
                img.save(path).unwrap();
            },
            2 => {
                let img = image::GrayAlphaImage::from_raw(img.width(), img.height(), buf).unwrap();
                img.save(path).unwrap();
            },
            3 => {
                let img = image::RgbImage::from_raw(img.width(), img.height(), buf).unwrap();
                img.save(path).unwrap();
            },
            4 => {
                let img = image::RgbaImage::from_raw(img.width(), img.height(), buf).unwrap();
                img.save(path).unwrap();
            },
            _ => panic!()
        }
    }

    /// Создаёт `TextureRef` на основе `ImageViewAbstract`.
    /// В основном используется для представления swapchain изображения в виде текстуры
    /// для вывода результата рендеринга
    pub fn from_vk_image_view(img: Arc<dyn ImageViewAbstract>, device: Arc<Device>) -> Result<Texture, String>
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
        Ok(Self{
            name : "".to_string(),

            _vk_image_dims: img_dims.into(),
            _vk_image_access: img.image(),
            _vk_image_view: img.clone(),
            _vk_sampler: sampler,
            _vk_device: device.clone(),

            _pix_fmt : pix_fmt,

            is_cubemap : match img.view_type() {
                TextureView::CubeArray | TextureView::Cube => true,
                _ => false
            },
            is_array : match img.view_type() {
                TextureView::CubeArray | TextureView::Dim1dArray | TextureView::Dim2dArray => true,
                _ => false
            },

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
        })
    }

    /*pub fn from_file_mutex<P : AsRef<std::path::Path> + ToString>(queue: Arc<Queue>, path: P) -> Result<TextureRef, String>
    {
        Ok(RcBox::construct(Self::from_file(queue, path)?))
    }*/

    /// Загрузка изображения-текстуры из файла.
    /// Поддерживаются форматы dds, ktx и все форматы, поддерживаемые crate'ом image
    pub fn from_file<P : AsRef<std::path::Path> + ToString>(queue: Arc<Queue>, path: P) -> Result<Texture, String>
    {
        let extension = path.as_ref().extension();
        let mut texture_builder = Texture::builder();
        texture_builder.mag_filter = TextureFilter::Linear;
        texture_builder.min_filter = TextureFilter::Linear;
        texture_builder.mip_mode = MipmapMode::Linear;
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

    /// Задаёт фильтрацию при сжатии изображения
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

    pub fn array_layers(&self) -> u32
    {
        self._vk_image_view.array_layers().count() as _
    }

    pub fn dims(&self) -> TextureDimensions
    {
        self._vk_image_dims
    }

    /// Обновление сэмплера текстуры
    pub fn update_sampler(&mut self)
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
                ..Default::default()
            }
        ).unwrap();
    }

    pub fn clear_color(&mut self, queue: Arc<Queue>)
    {
        let device = queue.device().clone();
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            device.clone(),
            queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();
        match self._pix_fmt.subpixels() {
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

    /*pub fn build_mutable_mutex(self, device: Arc<Device>, pix_fmt: TexturePixelFormat, dimensions: TextureDimensions) -> Result<TextureRef, String>
    {
        Ok(RcBox::construct(self.build_mutable(device, pix_fmt, dimensions)?))
    }*/

    /// Строит изменяемое изображение
    pub fn build_mutable(self, device: Arc<Device>, pix_fmt: TexturePixelFormat, dimensions: TextureDimensions) -> Result<Texture, String>
    {
        let properties = device.physical_device().format_properties(pix_fmt);
        let ffeatures = properties.potential_format_features();
        let usage = ImageUsage {
            transfer_source: ffeatures.transfer_src,
            transfer_destination: ffeatures.transfer_dst,
            sampled: ffeatures.sampled_image,
            storage: ffeatures.storage_image,
            color_attachment: ffeatures.color_attachment,
            depth_stencil_attachment: ffeatures.depth_stencil_attachment,
            transient_attachment: false,
            input_attachment: false,
        };
        
        let texture = AttachmentImage::with_usage(
            device.clone(),
            [dimensions.width(), dimensions.height()],
            pix_fmt.vk_format(),
            usage
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

        let image_view = ImageView::new_default(texture.clone()).unwrap();

        Ok(Texture {
            name: self.name.to_string(),
            _vk_image_dims: dimensions,
            _vk_image_view: image_view.clone(),
            _vk_image_access: texture.clone(),
            _vk_device: device.clone(),
            _pix_fmt: pix_fmt,
            _vk_sampler: sampler,
            
            is_cubemap : match image_view.view_type() {
                TextureView::CubeArray | TextureView::Cube => true,
                _ => false
            },
            is_array : match image_view.view_type() {
                TextureView::CubeArray | TextureView::Dim1dArray | TextureView::Dim2dArray => true,
                _ => false
            },

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
    
    /*pub fn build_immutable_compressed_mutex<Rdr : Read + Seek + BufRead>(self, reader: &mut Rdr, queue: Arc<Queue>) -> Result<TextureRef, String>
    {
        Ok(RcBox::construct(self.build_immutable_compressed(reader, queue)?))
    }*/

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
        let image_view = ImageView::new_default(texture.clone()).unwrap();
        Ok(Texture {
            name: self.name.clone(),
            _vk_image_dims: dimensions,
            _vk_image_view: image_view.clone(),
            _vk_image_access: texture.clone(),
            _vk_device: device.clone(),
            _pix_fmt: header.pixel_format(),
            _vk_sampler: sampler,
            
            is_cubemap : match image_view.view_type() {
                TextureView::CubeArray | TextureView::Cube => true,
                _ => false
            },
            is_array : match image_view.view_type() {
                TextureView::CubeArray | TextureView::Dim1dArray | TextureView::Dim2dArray => true,
                _ => false
            },

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

    /*pub fn build_immutable_mutex<Rdr : Read + Seek + BufRead>(self, reader: &mut Rdr, queue: Arc<Queue>) -> Result<TextureRef, String>
    {
        Ok(RcBox::construct(self.build_immutable(reader, queue)?))
    }*/

    /// Строит неизменяемое несжатое изображение
    pub fn build_immutable<Rdr : Read + Seek + BufRead>(mut self, reader: &mut Rdr, queue: Arc<Queue>) -> Result<Texture, String>
    {
        let img_rdr = ImageReader::new(reader).with_guessed_format();
        if img_rdr.is_err() {
            return Err(String::from("Неизвестный формат изображения"));
        }
        let img_rdr = img_rdr.unwrap();
        let image = img_rdr.decode().unwrap();

        let (image_data, pix_fmt, (width, height)) = match image {
            image::DynamicImage::ImageLuma8(ref img)    => (img.as_raw().as_bytes().to_vec(), Format::R8_UNORM, img.dimensions()),
            image::DynamicImage::ImageLumaA8(ref img)   => (img.as_raw().as_bytes().to_vec(), Format::R8G8_UNORM, img.dimensions()),
            image::DynamicImage::ImageRgb8(ref img)     => (image.to_rgba8().as_raw().as_bytes().to_vec(), Format::R8G8B8A8_UNORM, img.dimensions()),
            image::DynamicImage::ImageRgba8(ref img)    => (img.as_raw().as_bytes().to_vec(), Format::R8G8B8A8_UNORM, img.dimensions()),
            image::DynamicImage::ImageLuma16(ref img)   => (img.as_raw().as_bytes().to_vec(), Format::R16_UNORM, img.dimensions()),
            image::DynamicImage::ImageLumaA16(ref img)  => (img.as_raw().as_bytes().to_vec(), Format::R16G16_UNORM, img.dimensions()),
            image::DynamicImage::ImageRgb16(ref img)    => (img.as_raw().as_bytes().to_vec(), Format::R16G16B16_UNORM, img.dimensions()),
            image::DynamicImage::ImageRgba16(ref img)   => (img.as_raw().as_bytes().to_vec(), Format::R16G16B16A16_UNORM, img.dimensions()),
            image::DynamicImage::ImageRgb32F(ref img)   => (img.as_raw().as_bytes().to_vec(), Format::R32G32B32_SFLOAT, img.dimensions()),
            image::DynamicImage::ImageRgba32F(ref img)  => (img.as_raw().as_bytes().to_vec(), Format::R32G32B32A32_SFLOAT, img.dimensions()),
            _ => panic!("Неизвестный формат пикселей")
        };
        let dimensions = TextureDimensions::Dim2d { width, height, array_layers: 1 };

        let (texture, tex_future) = ImmutableImage::from_iter(
            image_data.iter().cloned(),
            dimensions,
            match self.mip_mode {
                MipmapMode::Linear  => MipmapsCount::Log2,
                MipmapMode::Nearest => MipmapsCount::One
            },
            pix_fmt,
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
        let image_view = ImageView::new_default(texture.clone()).unwrap();
        Ok(Texture {
            name: self.name.clone(),
            _vk_image_dims: dimensions,
            _vk_image_view: ImageView::new_default(texture.clone()).unwrap(),
            _vk_image_access: texture.clone(),
            _vk_device: queue.device().clone(),
            _pix_fmt: pix_fmt,
            _vk_sampler: sampler,
            
            is_cubemap : match image_view.view_type() {
                TextureView::CubeArray | TextureView::Cube => true,
                _ => false
            },
            is_array : match image_view.view_type() {
                TextureView::CubeArray | TextureView::Dim1dArray | TextureView::Dim2dArray => true,
                _ => false
            },
            
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
        
        let u64s = (format.block_size().unwrap() / 8) as u32;
        
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
        let size = self.pixel_format().block_size().unwrap();
        size as _
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
        let size = self.pixel_format().block_size().unwrap();
        size as _
    }
}