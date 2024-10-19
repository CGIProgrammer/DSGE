use super::pixel_format::TexturePixelFormat;
use byteorder::{BigEndian, ByteOrder, LittleEndian};

pub(crate) const KTX1_IDENTIFIER: [u8; 12] = [
    0xAB, 0x4B, 0x54, 0x58, 0x20, 0x31, 0x31, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
];
/*pub(crate) const KTX2_IDENTIFIER: [u8; 12] = [
    0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A
];

#[derive(Copy, Clone)]
pub struct Ktx2Header
{
    identifier: [u8; 12],
    vk_format: u32,
    type_size: u32,
    pixel_width: u32,
    pixel_height: u32,
    pixel_depth: u32,
    layer_count: u32,
    face_count: u32,
    level_count: u32,
    supercompression_scheme: u32,
    dfd_byte_offset: u32,
    dfd_byte_length: u32,
    kvd_byte_offset: u32,
    kvd_byte_length: u32,
    sgd_byte_offset: u64,
    sgd_byte_length: u64,
}

impl Ktx2Header
{
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String>
    {
        if size_of::<Ktx2Header>() != bytes.len() {
            return Err(format!("Длина байтового массива ({}) не равна {}",
                bytes.len(),
                size_of::<Ktx2Header>()
            ));
        }
        let header: Ktx2Header = unsafe {
            from_raw_parts(bytes as *const [u8] as _, 1)
        }[0];
        assert!(header.identifier == KTX2_IDENTIFIER);
        Ok(header)
    }

    pub fn vk_format(&self) -> u32
    {
        self.vk_format
    }

    #[inline]
    pub fn get_pixel_format(&self) -> TexturePixelFormat {

        match self.vk_format {
            super::pixel_format::GL_COMPRESSED_RGB8_ETC2      => TexturePixelFormat::ETC2_R8G8B8_SRGB_BLOCK,
            super::pixel_format::GL_COMPRESSED_RGBA8_ETC2_EAC => TexturePixelFormat::ETC2_R8G8B8A8_SRGB_BLOCK,
            super::pixel_format::GL_COMPRESSED_R11_EAC        => TexturePixelFormat::EAC_R11_UNORM_BLOCK,
            super::pixel_format::GL_COMPRESSED_RG11_EAC       => TexturePixelFormat::EAC_R11G11_UNORM_BLOCK,
            super::pixel_format::GL_COMPRESSED_RGB_S3TC_DXT1_EXT  => TexturePixelFormat::BC1_RGB_SRGB_BLOCK,
            super::pixel_format::GL_COMPRESSED_RGBA_S3TC_DXT1_EXT => TexturePixelFormat::BC1_RGBA_SRGB_BLOCK,
            super::pixel_format::GL_COMPRESSED_RGBA_S3TC_DXT3_EXT => TexturePixelFormat::BC3_SRGB_BLOCK,
            super::pixel_format::GL_COMPRESSED_RGBA_S3TC_DXT5_EXT => TexturePixelFormat::BC5_UNORM_BLOCK,
            _ => panic!("Неизвестный формат сжатия 0x{:X}", self.vk_format)
        }
    }
}*/

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KTXHeader {
    _big_endian: bool,
    _gl_type: u32,
    _gl_type_size: u32,
    _gl_format: u32,
    _gl_internal_format: u32,
    _gl_base_internal_format: u32,
    _pixel_width: u32,
    _pixel_height: u32,
    _pixel_depth: u32,
    _array_elements: u32,
    _faces: u32,
    _mipmap_levels: u32,
    _bytes_of_key_value_data: u32,
}

#[allow(dead_code)]
impl KTXHeader {
    /// Reads first 64 bytes to parse KTX header data, returns a `KtxHeader`.
    pub fn from_bytes(first_64_bytes: &[u8]) -> Result<Self, String> {
        debug_assert!(first_64_bytes.len() >= 64);
        debug_assert_eq!(&first_64_bytes[..12], &KTX1_IDENTIFIER, "Not KTX1");

        let big_endian = first_64_bytes[12] == 4;

        let mut vals: [u32; 12] = <_>::default();
        if big_endian {
            BigEndian::read_u32_into(&first_64_bytes[16..64], &mut vals);
        } else {
            LittleEndian::read_u32_into(&first_64_bytes[16..64], &mut vals);
        }

        Ok(Self {
            _big_endian: big_endian,
            _gl_type: vals[0],
            _gl_type_size: vals[1],
            _gl_format: vals[2],
            _gl_internal_format: vals[3],
            _gl_base_internal_format: vals[4],
            _pixel_width: vals[5],
            _pixel_height: vals[6],
            _pixel_depth: vals[7],
            _array_elements: vals[8],
            _faces: vals[9],
            _mipmap_levels: vals[10],
            _bytes_of_key_value_data: vals[11],
        })
    }
    #[inline]
    pub fn big_endian(&self) -> bool {
        self._big_endian
    }
    #[inline]
    pub fn gl_type(&self) -> u32 {
        self._gl_type
    }
    #[inline]
    pub fn gl_type_size(&self) -> u32 {
        self._gl_type_size
    }
    #[inline]
    pub fn gl_format(&self) -> u32 {
        self._gl_format
    }
    #[inline]
    pub fn get_pixel_format(&self) -> TexturePixelFormat {
        match self._gl_internal_format {
            super::pixel_format::GL_COMPRESSED_RGB8_ETC2 => {
                TexturePixelFormat::ETC2_R8G8B8_SRGB_BLOCK
            }
            super::pixel_format::GL_COMPRESSED_RGBA8_ETC2_EAC => {
                TexturePixelFormat::ETC2_R8G8B8A8_SRGB_BLOCK
            }
            super::pixel_format::GL_COMPRESSED_R11_EAC => TexturePixelFormat::EAC_R11_UNORM_BLOCK,
            super::pixel_format::GL_COMPRESSED_RG11_EAC => {
                TexturePixelFormat::EAC_R11G11_UNORM_BLOCK
            }
            super::pixel_format::GL_COMPRESSED_RGB_S3TC_DXT1_EXT => {
                TexturePixelFormat::BC1_RGB_SRGB_BLOCK
            }
            super::pixel_format::GL_COMPRESSED_RGBA_S3TC_DXT1_EXT => {
                TexturePixelFormat::BC1_RGBA_SRGB_BLOCK
            }
            super::pixel_format::GL_COMPRESSED_RGBA_S3TC_DXT3_EXT => {
                TexturePixelFormat::BC3_SRGB_BLOCK
            }
            super::pixel_format::GL_COMPRESSED_RGBA_S3TC_DXT5_EXT => {
                TexturePixelFormat::BC5_UNORM_BLOCK
            }
            _ => panic!("Неизвестный формат сжатия 0x{:X}", self._gl_internal_format),
        }
    }
    #[inline]
    pub fn gl_base_internal_format(&self) -> u32 {
        self._gl_base_internal_format
    }
    #[inline]
    pub fn pixel_width(&self) -> u32 {
        self._pixel_width
    }
    #[inline]
    pub fn pixel_height(&self) -> u32 {
        self._pixel_height
    }
    #[inline]
    pub fn pixel_depth(&self) -> u32 {
        self._pixel_depth
    }
    #[inline]
    pub fn array_elements(&self) -> u32 {
        self._array_elements
    }
    #[inline]
    pub fn faces(&self) -> u32 {
        self._faces
    }
    #[inline]
    pub fn mipmap_levels(&self) -> u32 {
        self._mipmap_levels
    }
    #[inline]
    pub fn bytes_of_key_value_data(&self) -> u32 {
        self._bytes_of_key_value_data
    }
}
