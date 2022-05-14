use byteorder::{ByteOrder, LittleEndian};
//use super::pixel_format::*;
use crate::texture::pixel_format::TexturePixelFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DDSPixFormat {
    _size : u32,
    _flags : u32,
    pub four_cc : u32,
    _rgb_bit_count : u32,
    _r_bit_mask : u32,
    _g_bit_mask : u32,
    _b_bit_mask : u32,
    _a_bit_mask : u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DDSHeader {
    pub signature : u32,
    _size : u32,
    _flags : u32,
    pub height : u32,
    pub width : u32,
    _pitch_or_linear_size : u32,
    _depth : u32,
    pub mip_map_count : u32,
    _reserved1 : [u32; 11],
    ddspf : DDSPixFormat,
    _caps1 : u32,
    _caps2 : u32,
    _caps3 : u32,
    _caps4 : u32,
    _reserved2 : u32,
}

impl DDSHeader
{
    pub fn from_bytes(first_128_bytes: &[u8]) -> Result<Self, String> {
        debug_assert!(first_128_bytes.len() >= 128);
        let mut vals: [u32; 32] = [0; 32];
        LittleEndian::read_u32_into(&first_128_bytes[0..128], &mut vals);
        if vals[0] != 0x20534444 {
            return Err(String::from("Не DDS файл"));
        }
        match vals[21] {
            0x31545844 => (),
            0x33545844 => (),
            0x35545844 => (),
            _ => {
                return Err(String::from("Неизвестный формат сжатия в DDS изображении"));
            }
        }
        Ok(Self {
            signature : vals[0],
            _size : vals[1],
            _flags : vals[2],
            height : vals[3],
            width : vals[4],
            _pitch_or_linear_size : vals[5],
            _depth : vals[6],
            mip_map_count : vals[7],
            _reserved1 : [0; 11],
            ddspf : DDSPixFormat {
                _size : vals[19],
                _flags : vals[20],
                four_cc : vals[21],
                _rgb_bit_count : vals[22],
                _r_bit_mask : vals[23],
                _g_bit_mask : vals[24],
                _b_bit_mask : vals[25],
                _a_bit_mask : vals[26],
            },
            _caps1 : vals[27],
            _caps2 : vals[28],
            _caps3 : vals[29],
            _caps4 : vals[30],
            _reserved2 : vals[31],
        })
    }

    pub fn get_pixel_format(&self) -> TexturePixelFormat
    {
        match self.ddspf.four_cc {
            0x31545844 => TexturePixelFormat::BC1_RGB_SRGB_BLOCK,//::S3tcDXT1,
            0x33545844 => TexturePixelFormat::BC3_SRGB_BLOCK,//::S3tcDXT3,
            0x35545844 => TexturePixelFormat::BC5_UNORM_BLOCK,//::S3tcDXT5,
            _ => panic!("Неизвестный формат сжатия в DDS изображении")
        }
    }
}