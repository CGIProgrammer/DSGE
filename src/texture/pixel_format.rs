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

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum TexturePixelFormat
{
    Gray8i = 0,
    Gray8u,
	Gray16i,
	Gray16u,
	Gray16f,
	Gray32i,
	Gray32u,
	Gray32f,
	Gray64f,

    RG8i,
    RG8u,
	RG16i,
	RG16u,
	RG16f,
	RG32i,
	RG32u,
	RG32f,
	RG64f,

    RGB8i,
    RGB8u,
	RGB16i,
	RGB16u,
	RGB16f,
	RGB32i,
	RGB32u,
	RGB32f,
	RGB64f,

    RGBA8i,
    RGBA8u,
	RGBA16i,
	RGBA16u,
	RGBA16f,
	RGBA32i,
	RGBA32u,
	RGBA32f,
	RGBA64f,

	SRGB8i,
	SRGBA8i,

	Depth16u,
	Depth24u,
	Depth32f,

    BGR8i,
    BGR8u,
	BGR16i,
	BGR16u,
	BGR16f,
	BGR32i,
	BGR32u,
	BGR32f,
	BGR64f,

    BGRA8i,
    BGRA8u,
	BGRA16i,
	BGRA16u,
	BGRA16f,
	BGRA32i,
	BGRA32u,
	BGRA32f,
	BGRA64f,

    S3tcDXT1a,
    S3tcDXT1,
    S3tcDXT3,
    S3tcDXT5,

    Etc2RGB,
    Etc2RGBA,
    EacR11,
    EacRG11
}

#[allow(dead_code)]
impl TexturePixelFormat
{
    /*pub fn get_sized_format(&self) -> GLenum
    {
        PIXEL_FORMAT_TABLE[*self as usize].0
    }

    pub fn get_unsized_format(&self) -> GLenum
    {
        PIXEL_FORMAT_TABLE[*self as usize].1
    }

    pub fn get_cpu_type(&self) -> GLenum
    {
        PIXEL_FORMAT_TABLE[*self as usize].2
    }*/

	pub fn vk_format(&self) -> Format
	{
		PIXEL_FORMAT_TABLE[*self as usize].0
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

#[allow(dead_code)]
const PIXEL_FORMAT_TABLE : [(Format, &str); 67] = [
    (Format::R8_SNORM, "Gray8i"),
	(Format::R8_UNORM, "Gray8u"),
	(Format::R16_SNORM, "Gray16i"),
	(Format::R16_UNORM, "Gray16u"),
	(Format::R16_SFLOAT, "Gray16f"),
    (Format::R32_SINT, "Gray32i"),
	(Format::R32_UINT, "Gray32u"),
	(Format::R32_SFLOAT, "Gray32f"),
	(Format::R64_SFLOAT, "Gray64f"),

    (Format::R8G8_SNORM, "RG8i"),
	(Format::R8G8_UNORM, "RG8u"),
	(Format::R16G16_SNORM, "RG16i"),
	(Format::R16G16_UNORM, "RG16u"),
	(Format::R16G16_SFLOAT, "RG16f"),
    (Format::R32G32_SINT, "RG32i"),
	(Format::R32G32_UINT, "RG32u"),
	(Format::R32G32_SFLOAT, "RG32f"),
	(Format::R64G64_SFLOAT, "RG64f"),
	
    (Format::R8G8B8_SNORM, "RGB8i"),
	(Format::R8G8B8_UNORM, "RGB8u"),
	(Format::R16G16B16_SNORM, "RGB16i"),
	(Format::R16G16B16_UNORM, "RGB16u"),
	(Format::R16G16B16_SFLOAT, "RGB16f"),
    (Format::R32G32B32_SINT, "RGB32i"),
	(Format::R32G32B32_UINT, "RGB32u"),
	(Format::R32G32B32_SFLOAT, "RGB32f"),
	(Format::R64G64B64_SFLOAT, "RGB32f"),

    (Format::R8G8B8A8_SNORM, "RGBA8i"),
	(Format::R8G8B8A8_UNORM, "RGBA8u"),
	(Format::R16G16B16A16_SNORM, "RGBA16i"),
	(Format::R16G16B16A16_UNORM, "RGBA16u"),
	(Format::R16G16B16A16_SFLOAT, "RGBA16f"),
    (Format::R32G32B32A32_SINT, "RGBA32i"),
	(Format::R32G32B32A32_UINT, "RGBA32u"),
	(Format::R32G32B32A32_SFLOAT, "RGBA32f"),
	(Format::R64G64B64A64_SFLOAT, "RGBA64f"),

	(Format::R8G8B8_SRGB, "SRGB8i"),
	(Format::R8G8B8A8_SRGB, "SRGBA8i"),

	(Format::D16_UNORM, "Depth16u"),
	(Format::D24_UNORM_S8_UINT, "Depth24u"),
	(Format::D32_SFLOAT, "Depth32f"),

    (Format::R8_UNORM, "BGR8i"),
	(Format::R8_UNORM, "BGR8u"),
	(Format::R8_UNORM, "BGR16i"),
	(Format::R8_UNORM, "BGR16u"),
	(Format::R8_UNORM, "BGR16f"),
    (Format::R8_UNORM, "BGR32i"),
	(Format::R8_UNORM, "BGR32u"),
	(Format::R8_UNORM, "BGR32f"),
	(Format::R8_UNORM, "BGR64f"),

    (Format::R8_UNORM, "BGRA8i"),
	(Format::R8_UNORM, "BGRA8u"),
	(Format::R8_UNORM, "BGRA16i"),
	(Format::R8_UNORM, "BGRA16u"),
	(Format::R8_UNORM, "BGRA16f"),
    (Format::R8_UNORM, "BGRA32i"),
	(Format::R8_UNORM, "BGRA32u"),
	(Format::R8_UNORM, "BGRA32f"),
	(Format::R8_UNORM, "BGRA64f"),

    (Format::BC1_RGBA_SRGB_BLOCK, "S3tcDXT1a"),
    (Format::BC1_RGB_SRGB_BLOCK, "S3tcDXT1"),
    (Format::BC2_SRGB_BLOCK, "S3tcDXT3"),
    (Format::BC3_SRGB_BLOCK, "S3tcDXT5"),

    (Format::ETC2_R8G8B8_SRGB_BLOCK, "Etc2RGB"),
    (Format::ETC2_R8G8B8A8_SRGB_BLOCK, "Etc2RGBA"),
    (Format::EAC_R11_UNORM_BLOCK, "R11Eac"),
    (Format::EAC_R11G11_UNORM_BLOCK, "RG11Eac"),
];