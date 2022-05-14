#[allow(dead_code)]

use crate::glenums::AttribType;

pub type GLenum = u32;
pub const GL_COMPRESSED_RGB8_ETC2 : GLenum = 0x9274;
pub const GL_COMPRESSED_RGBA8_ETC2_EAC : GLenum = 0x9278;
pub const GL_COMPRESSED_R11_EAC : GLenum = 0x9270;
pub const GL_COMPRESSED_RG11_EAC : GLenum = 0x9272;
pub const GL_COMPRESSED_RGB_S3TC_DXT1_EXT : GLenum =  0x83F0;
pub const GL_COMPRESSED_RGBA_S3TC_DXT1_EXT : GLenum =  0x83F1;
pub const GL_COMPRESSED_RGBA_S3TC_DXT3_EXT : GLenum =  0x83F2;
pub const GL_COMPRESSED_RGBA_S3TC_DXT5_EXT : GLenum =  0x83F3;

/*
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TextureSubpixelType
{
	SignedInteger = 1,
	UnsignedInteger = 2,
	Float = 3
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TextureSubpixelSize
{
	Byte = 1,
	Short = 2,
	TripleByte = 3,
	Basic = 4,
	Long = 8,
	LongLong = 16
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TextureCodingType
{
	Raw,
	Dxt1,
	Dxt1a,
	Dxt3,
	Dxt5,
	EtcEac
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TextureSubpixelSet
{
	Shadow,
	Gray,
	RG,
	RGB,
	BGR,
	RGBA,
	BGRA
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TPF
{
	pub set: TextureSubpixelSet,
	pub supbix_type: TextureSubpixelType,
	pub subpix_size: TextureSubpixelSize,
	pub coding: TextureCodingType,
}

impl TPF
{

}*/
use vulkano::format::*;
pub type TexturePixelFormat = Format;

pub trait TexturePixelFormatFeatures
{
	fn is_depth(&self) -> bool;
	fn subpixels(&self) -> u32;
	fn subpixel_size(&self) -> u32;
	fn size(&self) -> u32;
	fn from_vk_format(fmt: Format) -> Result<Self, String> where Self: Sized;
	fn vk_format(&self) -> Format;
}

impl TexturePixelFormatFeatures for TexturePixelFormat
{
	fn from_vk_format(fmt: Format) -> Result<Self, String>
	{
		Ok(fmt)
	}

	fn vk_format(&self) -> Format
	{
		*self
	}

	fn is_depth(&self) -> bool
	{
		self.type_depth().is_some()
	}

	fn subpixels(&self) -> u32
	{
		let cmps = self.components();
		cmps.iter().filter_map(|x|{if *x!=0 {Some(1)} else {None}}).count() as _
	}

	fn subpixel_size(&self) -> u32
	{
		self.components()[0] as _
	}

	fn size(&self) -> u32
	{
		self.block_size().unwrap() as _
	}
}