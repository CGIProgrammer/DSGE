/// Псевдонимы для типов `nalgebra`

use crate::texture::TexturePixelFormat;
use nalgebra;

pub trait NalgebraPixelType
{
    fn pix_fmt(&self) -> TexturePixelFormat;
}

pub type Vec2 = nalgebra::Vector2<f32>; impl NalgebraPixelType for Vec2 { fn pix_fmt(&self) -> TexturePixelFormat    { TexturePixelFormat::R32G32_SFLOAT }}
pub type Vec3 = nalgebra::Vector3<f32>; impl NalgebraPixelType for Vec3 { fn pix_fmt(&self) -> TexturePixelFormat    { TexturePixelFormat::R32G32B32_SFLOAT }}
pub type Vec4 = nalgebra::Vector4<f32>; impl NalgebraPixelType for Vec4 { fn pix_fmt(&self) -> TexturePixelFormat    { TexturePixelFormat::R32G32B32A32_SFLOAT }}
pub type BVec2 = nalgebra::Vector2<bool>; impl NalgebraPixelType for BVec2 { fn pix_fmt(&self) -> TexturePixelFormat { TexturePixelFormat::R8G8_UNORM }}
pub type BVec3 = nalgebra::Vector3<bool>; impl NalgebraPixelType for BVec3 { fn pix_fmt(&self) -> TexturePixelFormat { TexturePixelFormat::R8G8B8_UNORM }}
pub type BVec4 = nalgebra::Vector4<bool>; impl NalgebraPixelType for BVec4 { fn pix_fmt(&self) -> TexturePixelFormat { TexturePixelFormat::R8G8B8A8_UNORM }}
pub type DVec2 = nalgebra::Vector2<f64>; impl NalgebraPixelType for DVec2 { fn pix_fmt(&self) -> TexturePixelFormat  { TexturePixelFormat::R64G64_SFLOAT }}
pub type DVec3 = nalgebra::Vector3<f64>; impl NalgebraPixelType for DVec3 { fn pix_fmt(&self) -> TexturePixelFormat  { TexturePixelFormat::R64G64B64_SFLOAT }}
pub type DVec4 = nalgebra::Vector4<f64>; impl NalgebraPixelType for DVec4 { fn pix_fmt(&self) -> TexturePixelFormat  { TexturePixelFormat::R64G64B64A64_SFLOAT }}
pub type IVec2 = nalgebra::Vector2<i32>; impl NalgebraPixelType for IVec2 { fn pix_fmt(&self) -> TexturePixelFormat  { TexturePixelFormat::R8G8_SNORM }}
pub type IVec3 = nalgebra::Vector3<i32>; impl NalgebraPixelType for IVec3 { fn pix_fmt(&self) -> TexturePixelFormat  { TexturePixelFormat::R8G8B8_SNORM }}
pub type IVec4 = nalgebra::Vector4<i32>; impl NalgebraPixelType for IVec4 { fn pix_fmt(&self) -> TexturePixelFormat  { TexturePixelFormat::R8G8B8A8_SNORM }}
pub type UVec2 = nalgebra::Vector2<u32>; impl NalgebraPixelType for UVec2 { fn pix_fmt(&self) -> TexturePixelFormat  { TexturePixelFormat::R8G8_UNORM }}
pub type UVec3 = nalgebra::Vector3<u32>; impl NalgebraPixelType for UVec3 { fn pix_fmt(&self) -> TexturePixelFormat  { TexturePixelFormat::R8G8B8_UNORM }}
pub type UVec4 = nalgebra::Vector4<u32>; impl NalgebraPixelType for UVec4 { fn pix_fmt(&self) -> TexturePixelFormat  { TexturePixelFormat::R8G8B8A8_UNORM }}

#[allow(dead_code)]
pub type Mat2 = nalgebra::Matrix2<f32>;
#[allow(dead_code)]
pub type Mat3 = nalgebra::Matrix3<f32>;
#[allow(dead_code)]
pub type Mat4 = nalgebra::Matrix4<f32>;

#[allow(dead_code)]
pub type BMat2 = nalgebra::Matrix2<bool>;
#[allow(dead_code)]
pub type BMat3 = nalgebra::Matrix3<bool>;
#[allow(dead_code)]
pub type BMat4 = nalgebra::Matrix4<bool>;

#[allow(dead_code)]
pub type DMat2 = nalgebra::Matrix2<f64>;
#[allow(dead_code)]
pub type DMat3 = nalgebra::Matrix3<f64>;
#[allow(dead_code)]
pub type DMat4 = nalgebra::Matrix4<f64>;

#[allow(dead_code)]
pub type IMat2 = nalgebra::Matrix2<i32>;
#[allow(dead_code)]
pub type IMat3 = nalgebra::Matrix3<i32>;
#[allow(dead_code)]
pub type IMat4 = nalgebra::Matrix4<i32>;

#[allow(dead_code)]
pub type UMat2 = nalgebra::Matrix2<u32>;
#[allow(dead_code)]
pub type UMat3 = nalgebra::Matrix3<u32>;
#[allow(dead_code)]
pub type UMat4 = nalgebra::Matrix4<u32>;

/*unsafe impl VertexMember for Vec2
{
	
}*/

