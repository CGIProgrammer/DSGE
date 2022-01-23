#[allow(dead_code)]

use vulkano::format::Format;
pub type GLenum = u32;
pub const GL_COMPRESSED_RGB8_ETC2 : GLenum = 0x9274;
pub const GL_COMPRESSED_RGBA8_ETC2_EAC : GLenum = 0x9278;
pub const GL_COMPRESSED_R11_EAC : GLenum = 0x9270;
pub const GL_COMPRESSED_RG11_EAC : GLenum = 0x9272;
pub const GL_COMPRESSED_RGB_S3TC_DXT1_EXT : GLenum =  0x83F0;
pub const GL_COMPRESSED_RGBA_S3TC_DXT1_EXT : GLenum =  0x83F1;
pub const GL_COMPRESSED_RGBA_S3TC_DXT3_EXT : GLenum =  0x83F2;
pub const GL_COMPRESSED_RGBA_S3TC_DXT5_EXT : GLenum =  0x83F3;

/// Форматы текстур
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TexturePixelFormat
{
	// Однокомпонентные форматы

    Gray8i = 0,
    Gray8u,
	Gray16i,
	Gray16u,
	Gray16f,
	Gray32i,
	Gray32u,
	Gray32f,
	Gray64f,

	// 2-компонентные

    RG8i,
    RG8u,
	RG16i,
	RG16u,
	RG16f,
	RG32i,
	RG32u,
	RG32f,
	RG64f,

	// 3-компонентные

    RGB8i,
    RGB8u,
	RGB16i,
	RGB16u,
	RGB16f,
	RGB32i,
	RGB32u,
	RGB32f,
	RGB64f,

	// 4-компонентные

    RGBA8i,
    RGBA8u,
	RGBA16i,
	RGBA16u,
	RGBA16f,
	RGBA32i,
	RGBA32u,
	RGBA32f,
	RGBA64f,

	// SRGB

	SRGB8i,
	SRGBA8i,
	SBGR8,
	SBGRA8,

	// Глубина

	Depth16u,
	Depth24u,
	Depth32f,

	// 3 и 4 -компонентные в обратном порядке

    BGR8i,
    BGR8u,
    BGRA8i,
    BGRA8u,

	// Сжатые
	// S3TC
    S3tcDXT1a,
    S3tcDXT1,
    S3tcDXT3,
    S3tcDXT5,

	// ETC
    Etc2RGB,
    Etc2RGBA,
    EacR11,
    EacRG11
}

#[allow(dead_code)]
impl TexturePixelFormat
{
	pub fn vk_format(&self) -> Format
	{
		PIXEL_FORMAT_TABLE[*self as usize].0
	}

	pub fn from_vk_format(vk_format: Format) -> Result<TexturePixelFormat, String>
	{
		for (vk_fmt, _, tex_pix_fnt) in PIXEL_FORMAT_TABLE {
			if vk_fmt == vk_format {
				return Ok(tex_pix_fnt);
			}
		}
		Err(format!("Не поддерживаемый формат пикселя {:?}", vk_format))
	}

	pub fn is_compressed(&self) -> bool
	{
		*self as usize > 78
	}

    pub fn stringify(&self) -> &str
    {
        PIXEL_FORMAT_TABLE[*self as usize].1
    }
}

/// Таблица форматов
#[allow(dead_code)]
const PIXEL_FORMAT_TABLE : [(Format, &str, TexturePixelFormat); 55] = [
    (Format::R8_SNORM, "Gray8i", TexturePixelFormat::Gray8i),
	(Format::R8_UNORM, "Gray8u", TexturePixelFormat::Gray8u),
	(Format::R16_SNORM, "Gray16i", TexturePixelFormat::Gray16i),
	(Format::R16_UNORM, "Gray16u", TexturePixelFormat::Gray16u),
	(Format::R16_SFLOAT, "Gray16f", TexturePixelFormat::Gray16f),
    (Format::R32_SINT, "Gray32i", TexturePixelFormat::Gray32i),
	(Format::R32_UINT, "Gray32u", TexturePixelFormat::Gray32u),
	(Format::R32_SFLOAT, "Gray32f", TexturePixelFormat::Gray32f),
	(Format::R64_SFLOAT, "Gray64f", TexturePixelFormat::Gray64f),

    (Format::R8G8_SNORM, "RG8i", TexturePixelFormat::RG8i),
	(Format::R8G8_UNORM, "RG8u", TexturePixelFormat::RG8u),
	(Format::R16G16_SNORM, "RG16i", TexturePixelFormat::RG16i),
	(Format::R16G16_UNORM, "RG16u", TexturePixelFormat::RG16u),
	(Format::R16G16_SFLOAT, "RG16f", TexturePixelFormat::RG16f),
    (Format::R32G32_SINT, "RG32i", TexturePixelFormat::RG32i),
	(Format::R32G32_UINT, "RG32u", TexturePixelFormat::RG32u),
	(Format::R32G32_SFLOAT, "RG32f", TexturePixelFormat::RG32f),
	(Format::R64G64_SFLOAT, "RG64f", TexturePixelFormat::RG64f),
	
    (Format::R8G8B8_SNORM, "RGB8i", TexturePixelFormat::RGB8i),
	(Format::R8G8B8_UNORM, "RGB8u", TexturePixelFormat::RGB8u),
	(Format::R16G16B16_SNORM, "RGB16i", TexturePixelFormat::RGB16i),
	(Format::R16G16B16_UNORM, "RGB16u", TexturePixelFormat::RGB16u),
	(Format::R16G16B16_SFLOAT, "RGB16f", TexturePixelFormat::RGB16f),
    (Format::R32G32B32_SINT, "RGB32i", TexturePixelFormat::RGB32i),
	(Format::R32G32B32_UINT, "RGB32u", TexturePixelFormat::RGB32u),
	(Format::R32G32B32_SFLOAT, "RGB32f", TexturePixelFormat::RGB32f),
	(Format::R64G64B64_SFLOAT, "RGB32f", TexturePixelFormat::RGB32f),

    (Format::R8G8B8A8_SNORM, "RGBA8i", TexturePixelFormat::RGBA8i),
	(Format::R8G8B8A8_UNORM, "RGBA8u", TexturePixelFormat::RGBA8u),
	(Format::R16G16B16A16_SNORM, "RGBA16i", TexturePixelFormat::RGBA16i),
	(Format::R16G16B16A16_UNORM, "RGBA16u", TexturePixelFormat::RGBA16u),
	(Format::R16G16B16A16_SFLOAT, "RGBA16f", TexturePixelFormat::RGBA16f),
    (Format::R32G32B32A32_SINT, "RGBA32i", TexturePixelFormat::RGBA32i),
	(Format::R32G32B32A32_UINT, "RGBA32u", TexturePixelFormat::RGBA32u),
	(Format::R32G32B32A32_SFLOAT, "RGBA32f", TexturePixelFormat::RGBA32f),
	(Format::R64G64B64A64_SFLOAT, "RGBA64f", TexturePixelFormat::RGBA64f),

	(Format::R8G8B8_SRGB, "SRGB8i", TexturePixelFormat::SRGB8i),
	(Format::R8G8B8A8_SRGB, "SRGBA8i", TexturePixelFormat::SRGBA8i),
	(Format::B8G8R8_SRGB, "SBGR8", TexturePixelFormat::SBGR8),
	(Format::B8G8R8A8_SRGB, "SBGRA8", TexturePixelFormat::SBGRA8),


	(Format::D16_UNORM, "Depth16u", TexturePixelFormat::Depth16u),
	(Format::D24_UNORM_S8_UINT, "Depth24u", TexturePixelFormat::Depth24u),
	(Format::D32_SFLOAT, "Depth32f", TexturePixelFormat::Depth32f),

    (Format::B8G8R8_SNORM, "BGR8i", TexturePixelFormat::BGR8i),
	(Format::B8G8R8_UNORM, "BGR8u", TexturePixelFormat::BGR8u),

    (Format::B8G8R8A8_SNORM, "BGRA8i", TexturePixelFormat::BGRA8i),
	(Format::B8G8R8A8_UNORM, "BGRA8u", TexturePixelFormat::BGRA8u),

    (Format::BC1_RGBA_SRGB_BLOCK, "S3tcDXT1a", TexturePixelFormat::S3tcDXT1a),
    (Format::BC1_RGB_SRGB_BLOCK, "S3tcDXT1", TexturePixelFormat::S3tcDXT1),
    (Format::BC2_SRGB_BLOCK, "S3tcDXT3", TexturePixelFormat::S3tcDXT3),
    (Format::BC3_SRGB_BLOCK, "S3tcDXT5", TexturePixelFormat::S3tcDXT5),

    (Format::ETC2_R8G8B8_SRGB_BLOCK, "Etc2RGB", TexturePixelFormat::Etc2RGB),
    (Format::ETC2_R8G8B8A8_SRGB_BLOCK, "Etc2RGBA", TexturePixelFormat::Etc2RGBA),
    (Format::EAC_R11_UNORM_BLOCK, "EacR11", TexturePixelFormat::EacR11),
    (Format::EAC_R11G11_UNORM_BLOCK, "EacRG11", TexturePixelFormat::EacRG11),
];