use byteorder::{BigEndian, ByteOrder, LittleEndian};
use super::pixel_format::TexturePixelFormat;

pub(crate) const KTX1_IDENTIFIER: [u8; 12] = [
    0xAB, 0x4B, 0x54, 0x58, 0x20, 0x31, 0x31, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
];

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
            super::pixel_format::GL_COMPRESSED_RGB8_ETC2 => TexturePixelFormat::Etc2RGB,
            super::pixel_format::GL_COMPRESSED_RGBA8_ETC2_EAC => TexturePixelFormat::Etc2RGBA,
            super::pixel_format::GL_COMPRESSED_R11_EAC => TexturePixelFormat::EacR11,
            super::pixel_format::GL_COMPRESSED_RG11_EAC => TexturePixelFormat::EacRG11,
            super::pixel_format::GL_COMPRESSED_RGB_S3TC_DXT1_EXT => TexturePixelFormat::S3tcDXT1,
            super::pixel_format::GL_COMPRESSED_RGBA_S3TC_DXT1_EXT => TexturePixelFormat::S3tcDXT1a,
            super::pixel_format::GL_COMPRESSED_RGBA_S3TC_DXT3_EXT => TexturePixelFormat::S3tcDXT3,
            super::pixel_format::GL_COMPRESSED_RGBA_S3TC_DXT5_EXT => TexturePixelFormat::S3tcDXT5,
            _ => panic!("Неизвестный формат сжатия")
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
