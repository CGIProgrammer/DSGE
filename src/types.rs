/// Псевдонимы для типов `nalgebra`
use crate::texture::TexturePixelFormat;
use nalgebra::{self, Matrix4, Perspective3, RealField, Rotation3, Vector3, Vector4};

pub trait NalgebraPixelType {
    fn pix_fmt(&self) -> TexturePixelFormat;
}

pub type Vec2 = nalgebra::Vector2<f32>;
impl NalgebraPixelType for Vec2 {
    fn pix_fmt(&self) -> TexturePixelFormat {
        TexturePixelFormat::R32G32_SFLOAT
    }
}
pub type Vec3 = nalgebra::Vector3<f32>;
impl NalgebraPixelType for Vec3 {
    fn pix_fmt(&self) -> TexturePixelFormat {
        TexturePixelFormat::R32G32B32_SFLOAT
    }
}
pub type Vec4 = nalgebra::Vector4<f32>;
impl NalgebraPixelType for Vec4 {
    fn pix_fmt(&self) -> TexturePixelFormat {
        TexturePixelFormat::R32G32B32A32_SFLOAT
    }
}
pub type BVec2 = nalgebra::Vector2<bool>;
impl NalgebraPixelType for BVec2 {
    fn pix_fmt(&self) -> TexturePixelFormat {
        TexturePixelFormat::R8G8_UNORM
    }
}
pub type BVec3 = nalgebra::Vector3<bool>;
impl NalgebraPixelType for BVec3 {
    fn pix_fmt(&self) -> TexturePixelFormat {
        TexturePixelFormat::R8G8B8_UNORM
    }
}
pub type BVec4 = nalgebra::Vector4<bool>;
impl NalgebraPixelType for BVec4 {
    fn pix_fmt(&self) -> TexturePixelFormat {
        TexturePixelFormat::R8G8B8A8_UNORM
    }
}
pub type DVec2 = nalgebra::Vector2<f64>;
impl NalgebraPixelType for DVec2 {
    fn pix_fmt(&self) -> TexturePixelFormat {
        TexturePixelFormat::R64G64_SFLOAT
    }
}
pub type DVec3 = nalgebra::Vector3<f64>;
impl NalgebraPixelType for DVec3 {
    fn pix_fmt(&self) -> TexturePixelFormat {
        TexturePixelFormat::R64G64B64_SFLOAT
    }
}
pub type DVec4 = nalgebra::Vector4<f64>;
impl NalgebraPixelType for DVec4 {
    fn pix_fmt(&self) -> TexturePixelFormat {
        TexturePixelFormat::R64G64B64A64_SFLOAT
    }
}
pub type IVec2 = nalgebra::Vector2<i32>;
impl NalgebraPixelType for IVec2 {
    fn pix_fmt(&self) -> TexturePixelFormat {
        TexturePixelFormat::R8G8_SNORM
    }
}
pub type IVec3 = nalgebra::Vector3<i32>;
impl NalgebraPixelType for IVec3 {
    fn pix_fmt(&self) -> TexturePixelFormat {
        TexturePixelFormat::R8G8B8_SNORM
    }
}
pub type IVec4 = nalgebra::Vector4<i32>;
impl NalgebraPixelType for IVec4 {
    fn pix_fmt(&self) -> TexturePixelFormat {
        TexturePixelFormat::R8G8B8A8_SNORM
    }
}
pub type UVec2 = nalgebra::Vector2<u32>;
impl NalgebraPixelType for UVec2 {
    fn pix_fmt(&self) -> TexturePixelFormat {
        TexturePixelFormat::R8G8_UNORM
    }
}
pub type UVec3 = nalgebra::Vector3<u32>;
impl NalgebraPixelType for UVec3 {
    fn pix_fmt(&self) -> TexturePixelFormat {
        TexturePixelFormat::R8G8B8_UNORM
    }
}
pub type UVec4 = nalgebra::Vector4<u32>;
impl NalgebraPixelType for UVec4 {
    fn pix_fmt(&self) -> TexturePixelFormat {
        TexturePixelFormat::R8G8B8A8_UNORM
    }
}

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

pub trait ArrayInto {
    type T: RealField + Sized;
    fn into_mat4(self) -> Matrix4<Self::T>;
    fn x_direction(self) -> Vector3<Self::T>;
    fn y_direction(self) -> Vector3<Self::T>;
    fn z_direction(self) -> Vector3<Self::T>;
    fn vec3_location(self) -> Vector3<Self::T>;
    fn vec4_location(self) -> Vector4<Self::T>;
}

impl ArrayInto for Mat4 {
    type T = f32;
    
    #[inline(always)]
    fn into_mat4(self) -> Mat4 {
        self
    }

    #[inline(always)]
    fn vec3_location(self) -> Vec3 {
        self.column(3).xyz()
    }

    #[inline(always)]
    fn vec4_location(self) -> Vec4 {
        self.column(3).into()
    }

    #[inline(always)]
    fn x_direction(self) -> Vec3 {
        self.column(0).xyz()
    }

    #[inline(always)]
    fn y_direction(self) -> Vec3 {
        self.column(1).xyz()
    }

    #[inline(always)]
    fn z_direction(self) -> Vec3 {
        self.column(2).xyz()
    }
}

impl ArrayInto for [f32; 16] {
    type T = f32;

    #[inline(always)]
    fn into_mat4(self) -> Mat4 {
        unsafe { std::mem::transmute(self) }
    }

    #[inline(always)]
    fn vec3_location(self) -> Vec3 {
        self.into_mat4().column(3).xyz()
    }

    #[inline(always)]
    fn vec4_location(self) -> Vec4 {
        self.into_mat4().column(3).into()
    }

    #[inline(always)]
    fn x_direction(self) -> Vec3 {
        self.into_mat4().column(0).xyz()
    }

    #[inline(always)]
    fn y_direction(self) -> Vec3 {
        self.into_mat4().column(1).xyz()
    }

    #[inline(always)]
    fn z_direction(self) -> Vec3 {
        self.into_mat4().column(2).xyz()
    }
}

impl ArrayInto for [[f32; 4]; 4] {
    type T = f32;
    #[inline(always)]
    fn into_mat4(self) -> Mat4 {
        self.into()
    }

    #[inline(always)]
    fn vec3_location(self) -> Vec3 {
        self.into_mat4().column(3).xyz()
    }

    #[inline(always)]
    fn vec4_location(self) -> Vec4 {
        self.into_mat4().column(3).into()
    }

    #[inline(always)]
    fn x_direction(self) -> Vec3 {
        self.into_mat4().column(0).xyz()
    }

    #[inline(always)]
    fn y_direction(self) -> Vec3 {
        self.into_mat4().column(1).xyz()
    }

    #[inline(always)]
    fn z_direction(self) -> Vec3 {
        self.into_mat4().column(2).xyz()
    }
}

pub trait Transform3 {
    type T: RealField;
    fn rotation(&self) -> Rotation3<Self::T>;
    fn location(&self) -> Vector4<Self::T>;
    fn set_rotation(&mut self, rot: &Rotation3<Self::T>);
    fn rotate(&mut self, rot: &Rotation3<Self::T>);
    fn rotate_local(&mut self, rot: &Rotation3<Self::T>);
    fn rotated(&self, rot: &Rotation3<Self::T>) -> Self;
    fn rotated_local(&self, rot: &Rotation3<Self::T>) -> Self;
}

impl Transform3 for Mat4 {
    type T = f32;
    fn location(&self) -> Vector4<Self::T> {
        self.column(3).into()
    }

    fn rotation(&self) -> Rotation3<Self::T> {
        Rotation3::from_matrix(&Mat3::new(
            self[0], self[4], self[8], self[1], self[5], self[9], self[2], self[6], self[10],
        ))
    }

    fn set_rotation(&mut self, rot: &Rotation3<Self::T>) {
        self[(0, 0)] = rot[(0, 0)];
        self[(0, 1)] = rot[(0, 1)];
        self[(0, 2)] = rot[(0, 2)];
        self[(1, 0)] = rot[(1, 0)];
        self[(1, 1)] = rot[(1, 1)];
        self[(1, 2)] = rot[(1, 2)];
        self[(2, 0)] = rot[(2, 0)];
        self[(2, 1)] = rot[(2, 1)];
        self[(2, 2)] = rot[(2, 2)];
    }

    fn rotate(&mut self, rot: &Rotation3<Self::T>) {
        self.set_rotation(&(rot * self.rotation()));
    }

    fn rotate_local(&mut self, rot: &Rotation3<Self::T>) {
        self.set_rotation(&(self.rotation() * rot));
    }

    fn rotated(&self, rot: &Rotation3<Self::T>) -> Self {
        let mut result = *self;
        result.rotate(rot);
        result
    }

    fn rotated_local(&self, rot: &Rotation3<Self::T>) -> Self {
        let mut result = *self;
        result.rotate_local(rot);
        result
    }
}

pub trait FastProjection<T: RealField> {
    fn z_near(&self) -> T;
    fn z_far(&self) -> T;
    fn inv_z_near(&self) -> T;
    fn inv_z_far(&self) -> T;
    fn aspect_ratio(&self) -> T;
    fn fov_x(&self) -> T;
    fn fov_y(&self) -> T;
}

impl<T: RealField + Copy> FastProjection<T> for Perspective3<T> {
    fn z_far(&self) -> T {
        let mat = self.as_matrix();
        let i = mat[(2, 2)];
        let k = mat[(2, 3)];
        k / (i + T::one())
    }

    fn z_near(&self) -> T {
        let mat = self.as_matrix();
        let i = mat[(2, 2)];
        let k = mat[(2, 3)];
        k / (i - T::one())
    }

    fn inv_z_far(&self) -> T {
        self.z_far()
    }

    fn inv_z_near(&self) -> T {
        self.z_near()
    }

    fn aspect_ratio(&self) -> T {
        let mat = self.as_matrix();
        mat[(1, 1)] / mat[(0, 0)]
    }

    fn fov_x(&self) -> T {
        let mat = self.as_matrix();
        let result = (T::one() / mat[(0, 0)]).atan();
        result + result
    }

    fn fov_y(&self) -> T {
        let mat = self.as_matrix();
        let result = (T::one() / mat[(1, 1)]).atan();
        result + result
    }
}

impl<T: RealField + Copy> FastProjection<T> for Matrix4<T> {
    fn z_near(&self) -> T {
        let i = self[(2, 2)];
        let k = self[(2, 3)];
        k / (i + T::one())
    }

    fn z_far(&self) -> T {
        let i = self[(2, 2)];
        let k = self[(2, 3)];
        k / (i - T::one())
    }

    fn inv_z_near(&self) -> T {
        let i = self[(3, 2)];
        let k = self[(3, 3)];
        -T::one() / (i - k)
    }

    fn inv_z_far(&self) -> T {
        let i = self[(3, 2)];
        let k = self[(3, 3)];
        T::one() / (i + k)
    }

    fn aspect_ratio(&self) -> T {
        self[(1, 1)] / self[(0, 0)]
    }

    fn fov_x(&self) -> T {
        let result = (T::one() / self[(0, 0)]).atan();
        result + result
    }

    fn fov_y(&self) -> T {
        let result = (T::one() / self[(1, 1)]).atan();
        result + result
    }
}
