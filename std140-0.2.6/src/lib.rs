#![allow(non_camel_case_types)]

//! This module contains types that may be used to define Rust struct types that match the GLSL
//! std140 memory layout.
//!
//! Std140 is a standardized memory layout for GLSL shader interface blocks (e.g. uniform blocks).
//! An interface block is a group op typed GLSL variables. For details on the layout rules for
//! std140, please refer to the section 2.12.6.4 "Standard Uniform Block Layout" of the
//! [OpenGL ES 3.0 Specification](https://www.khronos.org/registry/OpenGL/specs/es/3.0/es_spec_3.0.pdf).
//!
//! This module aims to make it easy to construct and manipulate a block of std140 compatible memory
//! as a Rust struct, such that the struct's memory layout will match a GLSL interface block if
//! every block member is pairwise type-compatible with the struct field in the corresponding
//! position. Position here relates to the order in which block members and struct fields are
//! declared, e.g.: the 1st block member must be compatible with the 1st struct field, the 2nd block
//! member must be compatible with the 2nd struct field, etc. The struct itself has to be marked
//! with the [`#[repr_std140]`][repr_std140] attribute: this ensures the rust compiler will
//! correctly order and align the fields.
//!
//! For GLSL primitive types, compatibility is defined by the following mapping from GLSL types to
//! `std140` types:
//!
//! - `float`: [float]
//! - `vec2`: [vec2]
//! - `vec3`: [vec3]
//! - `vec4`: [vec4]
//! - `mat2`: [mat2x2][struct@mat2x2]
//! - `mat3`: [mat3x3][struct@mat3x3]
//! - `mat4`: [mat4x4][struct@mat4x4]
//! - `mat2x2`: [mat2x2][struct@mat2x2]
//! - `mat2x3`: [mat2x3][struct@mat2x3]
//! - `mat2x4`: [mat2x4][struct@mat2x4]
//! - `mat3x2`: [mat3x2][struct@mat3x2]
//! - `mat3x3`: [mat3x3][struct@mat3x3]
//! - `mat3x4`: [mat3x4][struct@mat3x4]
//! - `mat4x2`: [mat4x2][struct@mat4x2]
//! - `mat4x3`: [mat4x3][struct@mat4x3]
//! - `mat4x4`: [mat4x4][struct@mat4x4]
//! - `int`: [int]
//! - `ivec2`: [ivec2]
//! - `ivec3`: [ivec3]
//! - `ivec4`: [ivec4]
//! - `uint`: [uint]
//! - `uvec2`: [uvec2]
//! - `uvec3`: [uvec3]
//! - `uvec4`: [uvec4]
//! - `bool`: [boolean]
//! - `bvec2`: [bvec2]
//! - `bvec3`: [bvec3]
//! - `bvec4`: [bvec4]
//!
//! A GLSL struct type is compatible with a field if this field's type is a Rust struct marked with
//! [`#[repr_std140]`][repr_std140] and this struct's fields are pair-wise compatible with the GLSL
//! struct field in the corresponding position.
//!
//! A GLSL array of GLSL type `T` with compatible type `T_c` (as defined above) and length `N` is
//! compatible with a field of type `std140::array<T_c, N>`.
//!
//! # Example
//!
//! Given the following GLSL declaration of an (uniform) interface block:
//!
//! ```glsl
//! struct PointLight {
//!     vec3 position;
//!     float intensity;
//! }
//!
//! layout(std140) uniform Uniforms {
//!     mat4 transform;
//!     vec3 ambient_light_color;
//!     PointLight lights[2];
//! }
//! ```
//!
//! The following will produce a Rust struct instance with a compatible memory layout:
//!
//! ```rust
//! #[std140::repr_std140]
//! struct PointLight {
//!     position: std140::vec3,
//!     intensity: std140::float,
//! }
//!
//! #[std140::repr_std140]
//! struct Uniforms {
//!     transform: std140::mat4x4,
//!     ambient_light_color: std140::vec3,
//!     lights: std140::array<PointLight, 2>
//! }
//!
//! let instance = Uniforms {
//!     transform: std140::mat4x4(
//!         std140::vec4(1.0, 0.0, 0.0, 0.0),
//!         std140::vec4(0.0, 1.0, 0.0, 0.0),
//!         std140::vec4(0.0, 0.0, 1.0, 0.0),
//!         std140::vec4(0.0, 0.0, 0.0, 1.0),
//!     ),
//!     ambient_light_color: std140::vec3(0.2, 0.2, 0.2),
//!     lights: std140::array![
//!         PointLight {
//!             position: std140::vec3(10.0, 0.0, 10.0),
//!             intensity: std140::float(0.5)
//!         },
//!         PointLight {
//!             position: std140::vec3(0.0, 10.0, 10.0),
//!             intensity: std140::float(0.8)
//!         },
//!     ]
//! };
//! ```
//!
//! Note that although the field names match the block member names in this example, this is not
//! strictly necessary: only pairwise field-type compatibility is required.
//!
//! [repr_std140]: attr.repr_std140.html

use std::fmt;
use std::ops::{Deref, DerefMut, Index, IndexMut};

/// Attribute macro that can be applied to a struct to ensure its representation is compatible with
/// the std140 memory layout convention.
///
/// Can only be applied to a struct if all of its fields implement [ReprStd140].
///
/// Any struct marked with this attribute will automatically implement [Std140Struct]
///
/// # Example
///
/// ```rust
/// #[std140::repr_std140]
/// struct PointLight {
///     position: std140::vec3,
///     intensity: std140::float,
/// }
/// ```
pub use std140_macros::repr_std140;

/// Marker trait for types that can be used as fields in structs marked with
/// [`#[repr_std140]`][repr_std140].
///
/// [repr_std140]: attr.repr_std140.html
pub unsafe trait ReprStd140 {}

/// Marker trait for types that can be used as the element type for std140 [array][struct@array]s.
pub unsafe trait Std140ArrayElement: ReprStd140 {}

/// Marker trait for struct types that were marked with [`#[repr_std140]`][repr_std140].
///
/// [repr_std140]: attr.repr_std140.html
pub unsafe trait Std140Struct {}

unsafe impl<T> ReprStd140 for T where T: Std140Struct {}
unsafe impl<T> Std140ArrayElement for T where T: Std140Struct {}

/// Represents an std140 compatible array.
///
/// All elements in an std140 array are aligned to at least 16 bytes.
///
/// The [array!][macro@array] macro may be used to initialize an array.
///
/// # Example
///
/// ```
/// let std140_array: std140::array<std140::vec2, 2> = std140::array![
///     std140::vec2(1.0, 0.0),
///     std140::vec2(0.0, 1.0),
/// ];
/// ```
#[derive(Clone, Copy)]
pub struct array<T, const LEN: usize>
where
    T: Std140ArrayElement,
{
    internal: [ArrayElementWrapper<T>; LEN],
}

impl<T, const LEN: usize> array<T, { LEN }>
where
    T: Std140ArrayElement,
{
    #[doc(hidden)]
    pub fn from_wrapped(wrapped: [ArrayElementWrapper<T>; LEN]) -> Self {
        array { internal: wrapped }
    }
}

impl<T, const LEN: usize> PartialEq for array<T, { LEN }>
where
    T: Std140ArrayElement + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        for i in 0..LEN {
            if self.internal[i] != other.internal[i] {
                return false;
            }
        }

        true
    }
}

impl<T, const LEN: usize> fmt::Debug for array<T, { LEN }>
where
    T: Std140ArrayElement + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.internal.iter()).finish()
    }
}

// TODO: something like this? (if that ever becomes possible)
//impl<T, const LEN: usize> Unsize<slice<T>> for array<T, {LEN}> {}
//
//pub struct slice<T> where T: Std140ArrayElement {
//    internal: *mut [ArrayElementWrapper<T>]
//}

#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C, align(16))]
pub struct ArrayElementWrapper<T>
where
    T: Std140ArrayElement,
{
    pub element: T,
}

impl<T> fmt::Debug for ArrayElementWrapper<T>
where
    T: Std140ArrayElement + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <T as fmt::Debug>::fmt(&self.element, f)
    }
}

/// Initializes a `std140` [array][struct@array].
///
/// # Example
///
/// ```
/// let std140_array: std140::array<std140::vec2, 2> = std140::array![
///     std140::vec2(1.0, 0.0),
///     std140::vec2(0.0, 1.0),
/// ];
/// ```
#[macro_export]
macro_rules! array {
    ($elem:expr; $n:expr) => {
        $crate::array::from_wrapped([$crate::ArrayElementWrapper {
            element: $elem
        }; $n])
    };
    ($($x:expr),*) => {
        $crate::array::from_wrapped([
            $(
                $crate::ArrayElementWrapper {
                    element: $x
                }
            ),*
        ])
    };
    ($($x:expr,)*) => ($crate::array![$($x),*])
}

unsafe impl<T, const LEN: usize> ReprStd140 for array<T, { LEN }> where T: Std140ArrayElement {}

/// A 32-bit floating point value.
///
/// # Example
///
/// ```
/// let value = std140::float(0.5);
/// ```
#[repr(C, align(4))]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct float(pub f32);

unsafe impl ReprStd140 for float {}
unsafe impl Std140ArrayElement for float {}

impl From<f32> for float
{
    #[inline(always)]
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl From<f64> for float
{
    #[inline(always)]
    fn from(value: f64) -> Self {
        Self(value as f32)
    }
}

impl Into<f32> for float
{
    #[inline(always)]
    fn into(self) -> f32 {
        self.0
    }
}

impl Into<f64> for float
{
    #[inline(always)]
    fn into(self) -> f64 {
        self.0 as _
    }
}

/// A column vector of 2 [float] values.
///
/// # Example
///
/// ```
/// let value = std140::vec2(0.0, 1.0);
/// ```
#[repr(C, align(8))]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct vec2(pub f32, pub f32);

impl vec2 {
    /// Creates a new [vec2] with zeros in all positions.
    pub fn zero() -> Self {
        vec2(0.0, 0.0)
    }
}

unsafe impl ReprStd140 for vec2 {}
unsafe impl Std140ArrayElement for vec2 {}

impl Index<usize> for vec2 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for vec2 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl From<[f32; 2]> for vec2
{
    #[inline(always)]
    fn from(value: [f32; 2]) -> Self {
        Self(value[0], value[1])
    }
}

impl From<[f64; 2]> for vec2
{
    #[inline(always)]
    fn from(value: [f64; 2]) -> Self {
        Self(value[0] as _, value[1] as _)
    }
}

impl From<nalgebra::Vector2<f32>> for vec2
{
    #[inline(always)]
    fn from(value: nalgebra::Vector2<f32>) -> Self {
        Self(value.x, value.y)
    }
}

impl From<nalgebra::Vector2<f64>> for vec2
{
    #[inline(always)]
    fn from(value: nalgebra::Vector2<f64>) -> Self {
        Self(value.x as _, value.y as _)
    }
}

impl Into<[f32; 2]> for vec2
{
    #[inline(always)]
    fn into(self) -> [f32; 2] {
        [self.0, self.1]
    }
}

impl Into<[f64; 2]> for vec2
{
    #[inline(always)]
    fn into(self) -> [f64; 2] {
        [self.0 as _, self.1 as _]
    }
}

impl Into<nalgebra::Vector2<f32>> for vec2
{
    #[inline(always)]
    fn into(self) -> nalgebra::Vector2<f32> {
        nalgebra::Vector2::<f32>::new(self.0 as _, self.1 as _)
    }
}

impl Into<nalgebra::Vector2<f64>> for vec2
{
    #[inline(always)]
    fn into(self) -> nalgebra::Vector2<f64> {
        nalgebra::Vector2::<f64>::new(self.0 as _, self.1 as _)
    }
}

/// A column vector of 3 [float] values.
///
/// # Example
///
/// ```
/// let value = std140::vec3(0.0, 0.0, 1.0);
/// ```
#[repr(C, align(16))]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct vec3(pub f32, pub f32, pub f32);

impl vec3 {
    /// Creates a new [vec3] with zeros in all positions.
    pub fn zero() -> Self {
        vec3(0.0, 0.0, 0.0)
    }
}

unsafe impl ReprStd140 for vec3 {}
unsafe impl Std140ArrayElement for vec3 {}

impl Index<usize> for vec3 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for vec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl From<[f32; 3]> for vec3
{
    #[inline(always)]
    fn from(value: [f32; 3]) -> Self {
        Self(value[0], value[1], value[2])
    }
}

impl From<nalgebra::Vector3<f32>> for vec3
{
    #[inline(always)]
    fn from(value: nalgebra::Vector3<f32>) -> Self {
        Self(value.x, value.y, value.z)
    }
}

impl From<nalgebra::Vector3<f64>> for vec3
{
    #[inline(always)]
    fn from(value: nalgebra::Vector3<f64>) -> Self {
        Self(value.x as _, value.y as _, value.z as _)
    }
}

impl Into<[f32; 3]> for vec3
{
    #[inline(always)]
    fn into(self) -> [f32; 3] {
        [self.0, self.1, self.2]
    }
}

impl Into<[f64; 3]> for vec3
{
    #[inline(always)]
    fn into(self) -> [f64; 3] {
        [self.0 as _, self.1 as _, self.2 as _]
    }
}

impl Into<nalgebra::Vector3<f32>> for vec3
{
    #[inline(always)]
    fn into(self) -> nalgebra::Vector3<f32> {
        nalgebra::Vector3::<f32>::new(self.0 as _, self.1 as _, self.2 as _)
    }
}

impl Into<nalgebra::Vector3<f64>> for vec3
{
    #[inline(always)]
    fn into(self) -> nalgebra::Vector3<f64> {
        nalgebra::Vector3::<f64>::new(self.0 as _, self.1 as _, self.2 as _)
    }
}

/// A column vector of 4 [float] values.
///
/// # Example
///
/// ```
/// let value = std140::vec4(0.0, 0.0, 0.0, 1.0);
/// ```
#[repr(C, align(16))]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct vec4(pub f32, pub f32, pub f32, pub f32);

impl vec4 {
    /// Creates a new [vec4] with zeros in all positions.
    pub fn zero() -> Self {
        vec4(0.0, 0.0, 0.0, 0.0)
    }
}

unsafe impl ReprStd140 for vec4 {}
unsafe impl Std140ArrayElement for vec4 {}

impl Index<usize> for vec4 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            3 => &self.3,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for vec4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            3 => &mut self.3,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl From<[f32; 4]> for vec4
{
    #[inline(always)]
    fn from(value: [f32; 4]) -> Self {
        Self(value[0], value[1], value[2], value[3])
    }
}

impl From<nalgebra::Vector4<f32>> for vec4
{
    #[inline(always)]
    fn from(value: nalgebra::Vector4<f32>) -> Self {
        Self(value.x, value.y, value.z, value.w)
    }
}

impl From<nalgebra::Vector4<f64>> for vec4
{
    #[inline(always)]
    fn from(value: nalgebra::Vector4<f64>) -> Self {
        Self(value.x as _, value.y as _, value.z as _, value.w as _)
    }
}

impl Into<[f32; 4]> for vec4
{
    #[inline(always)]
    fn into(self) -> [f32; 4] {
        [self.0, self.1, self.2, self.3]
    }
}

impl Into<[f64; 4]> for vec4
{
    #[inline(always)]
    fn into(self) -> [f64; 4] {
        [self.0 as _, self.1 as _, self.2 as _, self.3 as _]
    }
}

impl Into<nalgebra::Vector4<f32>> for vec4
{
    #[inline(always)]
    fn into(self) -> nalgebra::Vector4<f32> {
        nalgebra::Vector4::<f32>::new(self.0 as _, self.1 as _, self.2 as _, self.3 as _)
    }
}

impl Into<nalgebra::Vector4<f64>> for vec4
{
    #[inline(always)]
    fn into(self) -> nalgebra::Vector4<f64> {
        nalgebra::Vector4::<f64>::new(self.0 as _, self.1 as _, self.2 as _, self.3 as _)
    }
}

/// A 32-bit signed integer value.
///
/// # Example
///
/// ```
/// let value = std140::int(1);
/// ```
#[repr(C, align(4))]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct int(pub i32);

unsafe impl ReprStd140 for int {}
unsafe impl Std140ArrayElement for int {}

impl From<i32> for int
{
    #[inline(always)]
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<i16> for int
{
    #[inline(always)]
    fn from(value: i16) -> Self {
        Self(value as _)
    }
}

impl From<i8> for int
{
    #[inline(always)]
    fn from(value: i8) -> Self {
        Self(value as _)
    }
}

impl Into<i32> for int
{
    #[inline(always)]
    fn into(self) -> i32 {
        self.0
    }
}

impl Into<i64> for int
{
    #[inline(always)]
    fn into(self) -> i64 {
        self.0 as _
    }
}

impl Into<i128> for int
{
    #[inline(always)]
    fn into(self) -> i128 {
        self.0 as _
    }
}

/// A column vector of 2 [int] values.
///
/// # Example
///
/// ```
/// let value = std140::ivec2(0, 1);
/// ```
#[repr(C, align(8))]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct ivec2(pub i32, pub i32);

impl ivec2 {
    /// Creates a new [ivec2] with zeros in all positions.
    pub fn zero() -> Self {
        ivec2(0, 0)
    }
}

unsafe impl ReprStd140 for ivec2 {}
unsafe impl Std140ArrayElement for ivec2 {}

impl Index<usize> for ivec2 {
    type Output = i32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for ivec2 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            _ => panic!("Index out of bounds"),
        }
    }
}

/// A column vector of 3 [int] values.
///
/// # Example
///
/// ```
/// let value = std140::ivec3(0, 0, 1);
/// ```
#[repr(C, align(16))]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct ivec3(pub i32, pub i32, pub i32);

impl ivec3 {
    /// Creates a new [ivec3] with zeros in all positions.
    pub fn zero() -> Self {
        ivec3(0, 0, 0)
    }
}

unsafe impl ReprStd140 for ivec3 {}
unsafe impl Std140ArrayElement for ivec3 {}

impl Index<usize> for ivec3 {
    type Output = i32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for ivec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            _ => panic!("Index out of bounds"),
        }
    }
}

/// A column vector of 4 [int] values.
///
/// # Example
///
/// ```
/// let value = std140::ivec4(0, 0, 0, 1);
/// ```
#[repr(C, align(16))]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct ivec4(pub i32, pub i32, pub i32, pub i32);

impl ivec4 {
    /// Creates a new [ivec4] with zeros in all positions.
    pub fn zero() -> Self {
        ivec4(0, 0, 0, 0)
    }
}

unsafe impl ReprStd140 for ivec4 {}
unsafe impl Std140ArrayElement for ivec4 {}

impl Index<usize> for ivec4 {
    type Output = i32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            3 => &self.3,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for ivec4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            3 => &mut self.3,
            _ => panic!("Index out of bounds"),
        }
    }
}

/// A 32-bit unsigned integer value.
///
/// # Example
///
/// ```
/// let value = std140::uint(1);
/// ```
#[repr(C, align(4))]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct uint(pub u32);

unsafe impl ReprStd140 for uint {}
unsafe impl Std140ArrayElement for uint {}

impl From<u32> for uint
{
    #[inline(always)]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<u16> for uint
{
    #[inline(always)]
    fn from(value: u16) -> Self {
        Self(value as _)
    }
}

impl From<u8> for uint
{
    #[inline(always)]
    fn from(value: u8) -> Self {
        Self(value as _)
    }
}

impl Into<u32> for uint
{
    #[inline(always)]
    fn into(self) -> u32 {
        self.0
    }
}

impl Into<u64> for uint
{
    #[inline(always)]
    fn into(self) -> u64 {
        self.0 as _
    }
}

impl Into<u128> for uint
{
    #[inline(always)]
    fn into(self) -> u128 {
        self.0 as _
    }
}

/// A column vector of 2 [uint] values.
///
/// # Example
///
/// ```
/// let value = std140::uvec2(0, 1);
/// ```
#[repr(C, align(8))]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct uvec2(pub u32, pub u32);

impl uvec2 {
    /// Creates a new [uvec2] with zeros in all positions.
    pub fn zero() -> Self {
        uvec2(0, 0)
    }
}

unsafe impl ReprStd140 for uvec2 {}
unsafe impl Std140ArrayElement for uvec2 {}

impl Index<usize> for uvec2 {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for uvec2 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            _ => panic!("Index out of bounds"),
        }
    }
}

/// A column vector of 3 [uint] values.
///
/// # Example
///
/// ```
/// let value = std140::uvec3(0, 0, 1);
/// ```
#[repr(C, align(16))]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct uvec3(pub u32, pub u32, pub u32);

impl uvec3 {
    /// Creates a new [uvec3] with zeros in all positions.
    pub fn zero() -> Self {
        uvec3(0, 0, 0)
    }
}

unsafe impl ReprStd140 for uvec3 {}
unsafe impl Std140ArrayElement for uvec3 {}

impl Index<usize> for uvec3 {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for uvec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            _ => panic!("Index out of bounds"),
        }
    }
}

/// A column vector of 4 [uint] values.
///
/// # Example
///
/// ```
/// let value = std140::uvec4(0, 0, 0, 1);
/// ```
#[repr(C, align(16))]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct uvec4(pub u32, pub u32, pub u32, pub u32);

impl uvec4 {
    /// Creates a new [uvec4] with zeros in all positions.
    pub fn zero() -> Self {
        uvec4(0, 0, 0, 0)
    }
}

unsafe impl ReprStd140 for uvec4 {}
unsafe impl Std140ArrayElement for uvec4 {}

impl Index<usize> for uvec4 {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            3 => &self.3,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for uvec4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            3 => &mut self.3,
            _ => panic!("Index out of bounds"),
        }
    }
}

/// A 32-bit boolean value.
///
/// [boolean::False] is stored identically to a [uint] of `0`; [boolean::True] is stored identically
/// to a [uint] of `1`.
///
/// # Example
///
/// ```
/// let value = std140::uint(1);
/// ```
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum boolean {
    True = 1,
    False = 0,
}

impl Default for boolean
{
    fn default() -> Self {
        boolean::False
    }
}

unsafe impl ReprStd140 for boolean {}
unsafe impl Std140ArrayElement for boolean {}

impl From<bool> for boolean {
    #[inline(always)]
    fn from(value: bool) -> Self {
        match value {
            true => boolean::True,
            false => boolean::False,
        }
    }
}

impl Into<bool> for boolean {
    #[inline(always)]
    fn into(self) -> bool {
        match self {
            boolean::True => true,
            boolean::False => false,
        }
    }
}

/// A column vector of 2 [boolean] values.
///
/// # Example
///
/// ```
/// let value = std140::bvec2(std140::boolean::False, std140::boolean::True);
/// ```
#[repr(C, align(8))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct bvec2(pub boolean, pub boolean);

impl Default for bvec2
{
    fn default() -> Self {
        bvec2(boolean::False, boolean::False)
    }
}
unsafe impl ReprStd140 for bvec2 {}
unsafe impl Std140ArrayElement for bvec2 {}

impl Index<usize> for bvec2 {
    type Output = boolean;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for bvec2 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            _ => panic!("Index out of bounds"),
        }
    }
}

/// A column vector of 3 [boolean] values.
///
/// # Example
///
/// ```
/// let value = std140::bvec3(std140::boolean::False, std140::boolean::False, std140::boolean::True);
/// ```
#[repr(C, align(16))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct bvec3(pub boolean, pub boolean, pub boolean);

impl Default for bvec3
{
    fn default() -> Self {
        bvec3(boolean::False, boolean::False, boolean::False)
    }
}
unsafe impl ReprStd140 for bvec3 {}
unsafe impl Std140ArrayElement for bvec3 {}

impl Index<usize> for bvec3 {
    type Output = boolean;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for bvec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            _ => panic!("Index out of bounds"),
        }
    }
}

/// A column vector of 4 [boolean] values.
///
/// # Example
///
/// ```
/// let value = std140::bvec4(
///     std140::boolean::False,
///     std140::boolean::False,
///     std140::boolean::False,
///     std140::boolean::True
/// );
/// ```
#[repr(C, align(16))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct bvec4(pub boolean, pub boolean, pub boolean, pub boolean);

impl Default for bvec4
{
    fn default() -> Self {
        bvec4(boolean::False, boolean::False, boolean::False, boolean::False)
    }
}
unsafe impl ReprStd140 for bvec4 {}
unsafe impl Std140ArrayElement for bvec4 {}

impl Index<usize> for bvec4 {
    type Output = boolean;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            3 => &self.3,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for bvec4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            3 => &mut self.3,
            _ => panic!("Index out of bounds"),
        }
    }
}

/// A 64-bit floating point value.
///
/// # Example
///
/// ```
/// let value = std140::double(0.5);
/// ```
#[repr(C, align(8))]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct double(pub f64);

unsafe impl ReprStd140 for double {}
unsafe impl Std140ArrayElement for double {}

impl From<f64> for double
{
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<f32> for double
{
    fn from(value: f32) -> Self {
        Self(value as _)
    }
}

impl Into<f64> for double
{
    #[inline(always)]
    fn into(self) -> f64 {
        self.0 as _
    }
}

impl Into<f32> for double
{
    #[inline(always)]
    fn into(self) -> f32 {
        self.0 as _
    }
}

/// A column vector of 2 [double] values.
///
/// # Example
///
/// ```
/// let value = std140::dvec2(0.0, 1.0);
/// ```
#[repr(C, align(16))]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct dvec2(pub f64, pub f64);

impl dvec2 {
    /// Creates a new [dvec2] with zeros in all positions.
    pub fn zero() -> Self {
        dvec2(0.0, 0.0)
    }
}

unsafe impl ReprStd140 for dvec2 {}
unsafe impl Std140ArrayElement for dvec2 {}

impl Index<usize> for dvec2 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for dvec2 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            _ => panic!("Index out of bounds"),
        }
    }
}

/// A column vector of 3 [double] values.
///
/// # Example
///
/// ```
/// let value = std140::dvec3(0.0, 0.0, 1.0);
/// ```
#[repr(C, align(32))]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct dvec3(pub f64, pub f64, pub f64);

impl dvec3 {
    /// Creates a new [dvec3] with zeros in all positions.
    pub fn zero() -> Self {
        dvec3(0.0, 0.0, 0.0)
    }
}

unsafe impl ReprStd140 for dvec3 {}
unsafe impl Std140ArrayElement for dvec3 {}

impl Index<usize> for dvec3 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for dvec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            _ => panic!("Index out of bounds"),
        }
    }
}

/// A column vector of 4 [double] values.
///
/// # Example
///
/// ```
/// let value = std140::dvec4(0.0, 0.0, 0.0, 1.0);
/// ```
#[repr(C, align(32))]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct dvec4(pub f64, pub f64, pub f64, pub f64);

impl dvec4 {
    /// Creates a new [dvec4] with zeros in all positions.
    pub fn zero() -> Self {
        dvec4(0.0, 0.0, 0.0, 0.0)
    }
}

unsafe impl ReprStd140 for dvec4 {}
unsafe impl Std140ArrayElement for dvec4 {}

impl Index<usize> for dvec4 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            3 => &self.3,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for dvec4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            3 => &mut self.3,
            _ => panic!("Index out of bounds"),
        }
    }
}

/// A matrix with 2 columns and 2 rows, represented by 2 [vec2] vectors.
///
/// # Example
///
/// ```
/// let value = std140::mat2x2(
///     std140::vec2(0.0, 1.0),
///     std140::vec2(0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct mat2x2 {
    columns: array<vec2, 2>,
}

impl Default for mat2x2
{
    fn default() -> Self {
        mat2x2(
            vec2(1.0, 0.0),
            vec2(0.0, 1.0),
        )
    }
}

impl mat2x2 {
    /// Creates a new [mat2x2] with zeros in all positions.
    pub fn zero() -> Self {
        mat2x2(vec2::zero(), vec2::zero())
    }
}

/// Initializes a [mat2x2][struct@mat2x2]
///
/// # Example
///
/// See [mat2x2][struct@mat2x2].
pub fn mat2x2(c0: vec2, c1: vec2) -> mat2x2 {
    mat2x2 {
        columns: array![c0, c1],
    }
}

unsafe impl ReprStd140 for mat2x2 {}
unsafe impl Std140ArrayElement for mat2x2 {}

impl Deref for mat2x2 {
    type Target = array<vec2, 2>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for mat2x2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for mat2x2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("mat2x2{:?}", &self.columns))
    }
}

impl From<[[f32; 2]; 2]> for mat2x2
{
    #[inline(always)]
    fn from(value: [[f32; 2]; 2]) -> Self {
        mat2x2(
            value[0].into(),
            value[1].into(),
        )
    }
}

/// A matrix with 2 columns and 3 rows, represented by 2 [vec3] vectors.
///
/// # Example
///
/// ```
/// let value = std140::mat2x3(
///     std140::vec3(0.0, 0.0, 1.0),
///     std140::vec3(0.0, 0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct mat2x3 {
    columns: array<vec3, 2>,
}

impl Default for mat2x3
{
    fn default() -> Self {
        Self::zero()
    }
}

impl mat2x3 {
    /// Creates a new [mat2x3] with zeros in all positions.
    pub fn zero() -> Self {
        mat2x3(vec3::zero(), vec3::zero())
    }
}

/// Initializes a [mat2x3][struct@mat2x3]
///
/// # Example
///
/// See [mat2x3][struct@mat2x3].
pub fn mat2x3(c0: vec3, c1: vec3) -> mat2x3 {
    mat2x3 {
        columns: array![c0, c1],
    }
}

unsafe impl ReprStd140 for mat2x3 {}
unsafe impl Std140ArrayElement for mat2x3 {}

impl Deref for mat2x3 {
    type Target = array<vec3, 2>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for mat2x3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for mat2x3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("mat2x3{:?}", &self.columns))
    }
}

/// A matrix with 2 columns and 4 rows, represented by 2 [vec4] vectors.
///
/// # Example
///
/// ```
/// let value = std140::mat2x4(
///     std140::vec4(0.0, 0.0, 0.0, 1.0),
///     std140::vec4(0.0, 0.0, 0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct mat2x4 {
    columns: array<vec4, 2>,
}

impl Default for mat2x4
{
    fn default() -> Self {
        Self::zero()
    }
}

impl mat2x4 {
    /// Creates a new [mat2x4] with zeros in all positions.
    pub fn zero() -> Self {
        mat2x4(vec4::zero(), vec4::zero())
    }
}

/// Initializes a [mat2x4][struct@mat2x4]
///
/// # Example
///
/// See [mat2x4][struct@mat2x4].
pub fn mat2x4(c0: vec4, c1: vec4) -> mat2x4 {
    mat2x4 {
        columns: array![c0, c1],
    }
}

unsafe impl ReprStd140 for mat2x4 {}
unsafe impl Std140ArrayElement for mat2x4 {}

impl Deref for mat2x4 {
    type Target = array<vec4, 2>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for mat2x4 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for mat2x4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("mat2x4{:?}", &self.columns))
    }
}

/// A matrix with 3 columns and 2 rows, represented by 3 [vec2] vectors.
///
/// # Example
///
/// ```
/// let value = std140::mat3x2(
///     std140::vec2(0.0, 1.0),
///     std140::vec2(0.0, 1.0),
///     std140::vec2(0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct mat3x2 {
    columns: array<vec2, 3>,
}

impl Default for mat3x2
{
    fn default() -> Self {
        Self::zero()
    }
}

impl mat3x2 {
    /// Creates a new [mat3x2] with zeros in all positions.
    pub fn zero() -> Self {
        mat3x2(vec2::zero(), vec2::zero(), vec2::zero())
    }
}

/// Initializes a [mat3x2][struct@mat3x2]
///
/// # Example
///
/// See [mat3x2][struct@mat3x2].
pub fn mat3x2(c0: vec2, c1: vec2, c2: vec2) -> mat3x2 {
    mat3x2 {
        columns: array![c0, c1, c2],
    }
}

unsafe impl ReprStd140 for mat3x2 {}
unsafe impl Std140ArrayElement for mat3x2 {}

impl Deref for mat3x2 {
    type Target = array<vec2, 3>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for mat3x2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for mat3x2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("mat3x2{:?}", &self.columns))
    }
}

/// A matrix with 3 columns and 3 rows, represented by 3 [vec3] vectors.
///
/// # Example
///
/// ```
/// let value = std140::mat3x3(
///     std140::vec3(0.0, 0.0, 1.0),
///     std140::vec3(0.0, 0.0, 1.0),
///     std140::vec3(0.0, 0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct mat3x3 {
    columns: array<vec3, 3>,
}

impl mat3x3 {
    /// Creates a new [mat3x3] with zeros in all positions.
    pub fn zero() -> Self {
        mat3x3(vec3::zero(), vec3::zero(), vec3::zero())
    }
}

/// Initializes a [mat3x3][struct@mat3x3]
///
/// # Example
///
/// See [mat3x3][struct@mat3x3].
pub fn mat3x3(c0: vec3, c1: vec3, c2: vec3) -> mat3x3 {
    mat3x3 {
        columns: array![c0, c1, c2],
    }
}

unsafe impl ReprStd140 for mat3x3 {}
unsafe impl Std140ArrayElement for mat3x3 {}

impl Deref for mat3x3 {
    type Target = array<vec3, 3>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for mat3x3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for mat3x3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("mat3x3{:?}", &self.columns))
    }
}

impl Default for mat3x3
{
    fn default() -> Self {
        mat3x3(
            vec3(1.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 0.0, 1.0),
        )
    }
}

impl From<[f32; 9]> for mat3x3
{
    #[inline(always)]
    fn from(value: [f32; 9]) -> Self {
        mat3x3(
            vec3(value[0], value[1], value[2]),
            vec3(value[3], value[4], value[5]),
            vec3(value[6], value[7], value[8]),
        )
    }
}

impl From<[[f32; 3]; 3]> for mat3x3
{
    #[inline(always)]
    fn from(value: [[f32; 3]; 3]) -> Self {
        mat3x3(
            value[0].into(),
            value[1].into(),
            value[2].into(),
        )
    }
}

impl From<nalgebra::Matrix3<f32>> for mat3x3
{
    #[inline(always)]
    fn from(value: nalgebra::Matrix3<f32>) -> Self {
        let value: [[f32; 3]; 3] = value.into();
        value.into()
    }
}

/*impl Into<[f32; 9]> for mat3x3
{
    #[inline(always)]
    fn into(self) -> [f32; 9] {
        mat3x3(
            vec3(value[0], value[1], value[2]),
            vec3(value[3], value[4], value[5]),
            vec3(value[6], value[7], value[8]),
        )
    }
}*/

impl Into<[[f32; 3]; 3]> for mat3x3
{
    #[inline(always)]
    fn into(self) -> [[f32; 3]; 3] {
        nalgebra::Matrix3::<f32>::from_columns(&[
            self.columns.internal[0].element.into(),
            self.columns.internal[1].element.into(),
            self.columns.internal[2].element.into(),
        ]).into()
    }
}

impl Into<nalgebra::Matrix3<f32>> for mat3x3
{
    #[inline(always)]
    fn into(self) -> nalgebra::Matrix3<f32> {
        nalgebra::Matrix3::<f32>::from_columns(&[
            self.columns.internal[0].element.into(),
            self.columns.internal[1].element.into(),
            self.columns.internal[2].element.into(),
        ])
    }
}

/// A matrix with 3 columns and 4 rows, represented by 3 [vec4] vectors.
///
/// # Example
///
/// ```
/// let value = std140::mat3x4(
///     std140::vec4(0.0, 0.0, 0.0, 1.0),
///     std140::vec4(0.0, 0.0, 0.0, 1.0),
///     std140::vec4(0.0, 0.0, 0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct mat3x4 {
    columns: array<vec4, 3>,
}

impl Default for mat3x4
{
    fn default() -> Self {
        Self::zero()
    }
}

impl mat3x4 {
    /// Creates a new [mat3x4] with zeros in all positions.
    pub fn zero() -> Self {
        mat3x4(vec4::zero(), vec4::zero(), vec4::zero())
    }
}

/// Initializes a [mat3x4][struct@mat3x4]
///
/// # Example
///
/// See [mat3x4][struct@mat3x4].
pub fn mat3x4(c0: vec4, c1: vec4, c2: vec4) -> mat3x4 {
    mat3x4 {
        columns: array![c0, c1, c2],
    }
}

unsafe impl ReprStd140 for mat3x4 {}
unsafe impl Std140ArrayElement for mat3x4 {}

impl Deref for mat3x4 {
    type Target = array<vec4, 3>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for mat3x4 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for mat3x4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("mat3x4{:?}", &self.columns))
    }
}

/// A matrix with 4 columns and 2 rows, represented by 4 [vec2] vectors.
///
/// # Example
///
/// ```
/// let value = std140::mat4x2(
///     std140::vec2(0.0, 1.0),
///     std140::vec2(0.0, 1.0),
///     std140::vec2(0.0, 1.0),
///     std140::vec2(0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct mat4x2 {
    columns: array<vec2, 4>,
}

impl Default for mat4x2
{
    fn default() -> Self {
        Self::zero()
    }
}

impl mat4x2 {
    /// Creates a new [mat4x2] with zeros in all positions.
    pub fn zero() -> Self {
        mat4x2(vec2::zero(), vec2::zero(), vec2::zero(), vec2::zero())
    }
}

/// Initializes a [mat4x2][struct@mat4x2]
///
/// # Example
///
/// See [mat4x2][struct@mat4x2].
pub fn mat4x2(c0: vec2, c1: vec2, c2: vec2, c3: vec2) -> mat4x2 {
    mat4x2 {
        columns: array![c0, c1, c2, c3],
    }
}

unsafe impl ReprStd140 for mat4x2 {}
unsafe impl Std140ArrayElement for mat4x2 {}

impl Deref for mat4x2 {
    type Target = array<vec2, 4>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for mat4x2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for mat4x2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("mat4x2{:?}", &self.columns))
    }
}

/// A matrix with 4 columns and 3 rows, represented by 4 [vec3] vectors.
///
/// # Example
///
/// ```
/// let value = std140::mat4x3(
///     std140::vec3(0.0, 0.0, 1.0),
///     std140::vec3(0.0, 0.0, 1.0),
///     std140::vec3(0.0, 0.0, 1.0),
///     std140::vec3(0.0, 0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct mat4x3 {
    columns: array<vec3, 4>,
}

impl Default for mat4x3
{
    fn default() -> Self {
        Self::zero()
    }
}

impl mat4x3 {
    /// Creates a new [mat4x3] with zeros in all positions.
    pub fn zero() -> Self {
        mat4x3(vec3::zero(), vec3::zero(), vec3::zero(), vec3::zero())
    }
}

/// Initializes a [mat4x3][struct@mat4x3]
///
/// # Example
///
/// See [mat4x3][struct@mat4x3].
pub fn mat4x3(c0: vec3, c1: vec3, c2: vec3, c3: vec3) -> mat4x3 {
    mat4x3 {
        columns: array![c0, c1, c2, c3],
    }
}

unsafe impl ReprStd140 for mat4x3 {}
unsafe impl Std140ArrayElement for mat4x3 {}

impl Deref for mat4x3 {
    type Target = array<vec3, 4>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for mat4x3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for mat4x3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("mat4x3{:?}", &self.columns))
    }
}

/// A matrix with 4 columns and 4 rows, represented by 4 [vec4] vectors.
///
/// # Example
///
/// ```
/// let value = std140::mat4x4(
///     std140::vec4(0.0, 0.0, 0.0, 1.0),
///     std140::vec4(0.0, 0.0, 0.0, 1.0),
///     std140::vec4(0.0, 0.0, 0.0, 1.0),
///     std140::vec4(0.0, 0.0, 0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct mat4x4 {
    columns: array<vec4, 4>,
}

impl mat4x4 {
    /// Creates a new [mat4x4] with zeros in all positions.
    pub fn zero() -> Self {
        mat4x4(vec4::zero(), vec4::zero(), vec4::zero(), vec4::zero())
    }
    pub fn identity() -> Self {
        mat4x4(
            vec4(1.0, 0.0, 0.0, 0.0),
            vec4(0.0, 1.0, 0.0, 0.0),
            vec4(0.0, 0.0, 1.0, 0.0),
            vec4(0.0, 0.0, 0.0, 1.0),
        )
    }
}

/// Initializes a [mat4x4][struct@mat4x4]
///
/// # Example
///
/// See [mat4x4][struct@mat4x4].
pub fn mat4x4(c0: vec4, c1: vec4, c2: vec4, c3: vec4) -> mat4x4 {
    mat4x4 {
        columns: array![c0, c1, c2, c3],
    }
}

unsafe impl ReprStd140 for mat4x4 {}
unsafe impl Std140ArrayElement for mat4x4 {}

impl Deref for mat4x4 {
    type Target = array<vec4, 4>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for mat4x4 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for mat4x4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("mat4x4{:?}", &self.columns))
    }
}

impl Default for mat4x4
{
    fn default() -> Self {
        Self::identity()
    }
}

impl From<[f32; 16]> for mat4x4
{
    #[inline(always)]
    fn from(value: [f32; 16]) -> Self {
        unsafe{std::mem::transmute(value)}
    }
}

impl From<[[f32; 4]; 4]> for mat4x4
{
    #[inline(always)]
    fn from(value: [[f32; 4]; 4]) -> Self {
        unsafe{std::mem::transmute(value)}
    }
}

impl From<nalgebra::Matrix4<f32>> for mat4x4
{
    #[inline(always)]
    fn from(value: nalgebra::Matrix4<f32>) -> Self {
        unsafe{std::mem::transmute(value)}
    }
}

impl Into<[f32; 16]> for mat4x4
{
    #[inline(always)]
    fn into(self) -> [f32; 16] {
        unsafe{std::mem::transmute(self)}
    }
}

impl Into<[[f32; 4]; 4]> for mat4x4
{
    #[inline(always)]
    fn into(self) -> [[f32; 4]; 4] {
        unsafe{std::mem::transmute(self)}
    }
}

impl Into<nalgebra::Matrix4<f32>> for mat4x4
{
    #[inline(always)]
    fn into(self) -> nalgebra::Matrix4<f32> {
        unsafe{std::mem::transmute(self)}
    }
}

/// A matrix with 2 columns and 2 rows, represented by 2 [dvec2] vectors.
///
/// # Example
///
/// ```
/// let value = std140::dmat2x2(
///     std140::dvec2(0.0, 1.0),
///     std140::dvec2(0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct dmat2x2 {
    columns: array<dvec2, 2>,
}

impl Default for dmat2x2
{
    fn default() -> Self {
        Self::zero()
    }
}

impl dmat2x2 {
    /// Creates a new [dmat2x2] with zeros in all positions.
    pub fn zero() -> Self {
        dmat2x2(dvec2::zero(), dvec2::zero())
    }
}

/// Initializes a [dmat2x2][struct@dmat2x2]
///
/// # Example
///
/// See [dmat2x2][struct@dmat2x2].
pub fn dmat2x2(c0: dvec2, c1: dvec2) -> dmat2x2 {
    dmat2x2 {
        columns: array![c0, c1],
    }
}

unsafe impl ReprStd140 for dmat2x2 {}
unsafe impl Std140ArrayElement for dmat2x2 {}

impl Deref for dmat2x2 {
    type Target = array<dvec2, 2>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for dmat2x2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for dmat2x2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("dmat2x2{:?}", &self.columns))
    }
}

/// A matrix with 2 columns and 3 rows, represented by 2 [dvec3] vectors.
///
/// # Example
///
/// ```
/// let value = std140::dmat2x3(
///     std140::dvec3(0.0, 0.0, 1.0),
///     std140::dvec3(0.0, 0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct dmat2x3 {
    columns: array<dvec3, 2>,
}

impl Default for dmat2x3
{
    fn default() -> Self {
        Self::zero()
    }
}

impl dmat2x3 {
    /// Creates a new [dmat2x3] with zeros in all positions.
    pub fn zero() -> Self {
        dmat2x3(dvec3::zero(), dvec3::zero())
    }
}

/// Initializes a [dmat2x3][struct@dmat2x3]
///
/// # Example
///
/// See [dmat2x3][struct@dmat2x3].
pub fn dmat2x3(c0: dvec3, c1: dvec3) -> dmat2x3 {
    dmat2x3 {
        columns: array![c0, c1],
    }
}

unsafe impl ReprStd140 for dmat2x3 {}
unsafe impl Std140ArrayElement for dmat2x3 {}

impl Deref for dmat2x3 {
    type Target = array<dvec3, 2>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for dmat2x3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for dmat2x3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("dmat2x3{:?}", &self.columns))
    }
}

/// A matrix with 2 columns and 4 rows, represented by 2 [dvec4] vectors.
///
/// # Example
///
/// ```
/// let value = std140::dmat2x4(
///     std140::dvec4(0.0, 0.0, 0.0, 1.0),
///     std140::dvec4(0.0, 0.0, 0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct dmat2x4 {
    columns: array<dvec4, 2>,
}

impl Default for dmat2x4
{
    fn default() -> Self {
        Self::zero()
    }
}

impl dmat2x4 {
    /// Creates a new [dmat2x4] with zeros in all positions.
    pub fn zero() -> Self {
        dmat2x4(dvec4::zero(), dvec4::zero())
    }
}

/// Initializes a [dmat2x4][struct@dmat2x4]
///
/// # Example
///
/// See [dmat2x4][struct@dmat2x4].
pub fn dmat2x4(c0: dvec4, c1: dvec4) -> dmat2x4 {
    dmat2x4 {
        columns: array![c0, c1],
    }
}

unsafe impl ReprStd140 for dmat2x4 {}
unsafe impl Std140ArrayElement for dmat2x4 {}

impl Deref for dmat2x4 {
    type Target = array<dvec4, 2>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for dmat2x4 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for dmat2x4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("dmat2x4{:?}", &self.columns))
    }
}

/// A matrix with 3 columns and 2 rows, represented by 3 [dvec2] vectors.
///
/// # Example
///
/// ```
/// let value = std140::dmat3x2(
///     std140::dvec2(0.0, 1.0),
///     std140::dvec2(0.0, 1.0),
///     std140::dvec2(0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct dmat3x2 {
    columns: array<dvec2, 3>,
}

impl Default for dmat3x2
{
    fn default() -> Self {
        Self::zero()
    }
}

impl dmat3x2 {
    /// Creates a new [dmat3x2] with zeros in all positions.
    pub fn zero() -> Self {
        dmat3x2(dvec2::zero(), dvec2::zero(), dvec2::zero())
    }
}

/// Initializes a [dmat3x2][struct@dmat3x2]
///
/// # Example
///
/// See [dmat3x2][struct@dmat3x2].
pub fn dmat3x2(c0: dvec2, c1: dvec2, c2: dvec2) -> dmat3x2 {
    dmat3x2 {
        columns: array![c0, c1, c2],
    }
}

unsafe impl ReprStd140 for dmat3x2 {}
unsafe impl Std140ArrayElement for dmat3x2 {}

impl Deref for dmat3x2 {
    type Target = array<dvec2, 3>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for dmat3x2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for dmat3x2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("dmat3x2{:?}", &self.columns))
    }
}

/// A matrix with 3 columns and 3 rows, represented by 3 [dvec3] vectors.
///
/// # Example
///
/// ```
/// let value = std140::dmat3x3(
///     std140::dvec3(0.0, 0.0, 1.0),
///     std140::dvec3(0.0, 0.0, 1.0),
///     std140::dvec3(0.0, 0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct dmat3x3 {
    columns: array<dvec3, 3>,
}

impl Default for dmat3x3
{
    fn default() -> Self {
        Self::zero()
    }
}

impl dmat3x3 {
    /// Creates a new [dmat3x3] with zeros in all positions.
    pub fn zero() -> Self {
        dmat3x3(dvec3::zero(), dvec3::zero(), dvec3::zero())
    }
}

/// Initializes a [dmat3x3][struct@dmat3x3]
///
/// # Example
///
/// See [dmat3x3][struct@dmat3x3].
pub fn dmat3x3(c0: dvec3, c1: dvec3, c2: dvec3) -> dmat3x3 {
    dmat3x3 {
        columns: array![c0, c1, c2],
    }
}

unsafe impl ReprStd140 for dmat3x3 {}
unsafe impl Std140ArrayElement for dmat3x3 {}

impl Deref for dmat3x3 {
    type Target = array<dvec3, 3>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for dmat3x3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for dmat3x3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("dmat3x3{:?}", &self.columns))
    }
}

/// A matrix with 3 columns and 4 rows, represented by 3 [dvec4] vectors.
///
/// # Example
///
/// ```
/// let value = std140::dmat3x4(
///     std140::dvec4(0.0, 0.0, 0.0, 1.0),
///     std140::dvec4(0.0, 0.0, 0.0, 1.0),
///     std140::dvec4(0.0, 0.0, 0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct dmat3x4 {
    columns: array<dvec4, 3>,
}

impl Default for dmat3x4
{
    fn default() -> Self {
        Self::zero()
    }
}

impl dmat3x4 {
    /// Creates a new [dmat3x4] with zeros in all positions.
    pub fn zero() -> Self {
        dmat3x4(dvec4::zero(), dvec4::zero(), dvec4::zero())
    }
}

/// Initializes a [dmat3x4][struct@dmat3x4]
///
/// # Example
///
/// See [dmat3x4][struct@dmat3x4].
pub fn dmat3x4(c0: dvec4, c1: dvec4, c2: dvec4) -> dmat3x4 {
    dmat3x4 {
        columns: array![c0, c1, c2],
    }
}

unsafe impl ReprStd140 for dmat3x4 {}
unsafe impl Std140ArrayElement for dmat3x4 {}

impl Deref for dmat3x4 {
    type Target = array<dvec4, 3>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for dmat3x4 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for dmat3x4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("dmat3x4{:?}", &self.columns))
    }
}

/// A matrix with 4 columns and 2 rows, represented by 4 [dvec2] vectors.
///
/// # Example
///
/// ```
/// let value = std140::dmat4x2(
///     std140::dvec2(0.0, 1.0),
///     std140::dvec2(0.0, 1.0),
///     std140::dvec2(0.0, 1.0),
///     std140::dvec2(0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct dmat4x2 {
    columns: array<dvec2, 4>,
}

impl Default for dmat4x2
{
    fn default() -> Self {
        Self::zero()
    }
}

impl dmat4x2 {
    /// Creates a new [dmat4x2] with zeros in all positions.
    pub fn zero() -> Self {
        dmat4x2(dvec2::zero(), dvec2::zero(), dvec2::zero(), dvec2::zero())
    }
}

/// Initializes a [dmat4x2][struct@dmat4x2]
///
/// # Example
///
/// See [dmat4x2][struct@dmat4x2].
pub fn dmat4x2(c0: dvec2, c1: dvec2, c2: dvec2, c3: dvec2) -> dmat4x2 {
    dmat4x2 {
        columns: array![c0, c1, c2, c3],
    }
}

unsafe impl ReprStd140 for dmat4x2 {}
unsafe impl Std140ArrayElement for dmat4x2 {}

impl Deref for dmat4x2 {
    type Target = array<dvec2, 4>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for dmat4x2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for dmat4x2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("dmat4x2{:?}", &self.columns))
    }
}

/// A matrix with 4 columns and 3 rows, represented by 4 [dvec3] vectors.
///
/// # Example
///
/// ```
/// let value = std140::dmat4x3(
///     std140::dvec3(0.0, 0.0, 1.0),
///     std140::dvec3(0.0, 0.0, 1.0),
///     std140::dvec3(0.0, 0.0, 1.0),
///     std140::dvec3(0.0, 0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct dmat4x3 {
    columns: array<dvec3, 4>,
}

impl Default for dmat4x3
{
    fn default() -> Self {
        Self::zero()
    }
}

impl dmat4x3 {
    /// Creates a new [dmat4x3] with zeros in all positions.
    pub fn zero() -> Self {
        dmat4x3(dvec3::zero(), dvec3::zero(), dvec3::zero(), dvec3::zero())
    }
}

/// Initializes a [dmat4x3][struct@dmat4x3]
///
/// # Example
///
/// See [dmat4x3][struct@dmat4x3].
pub fn dmat4x3(c0: dvec3, c1: dvec3, c2: dvec3, c3: dvec3) -> dmat4x3 {
    dmat4x3 {
        columns: array![c0, c1, c2, c3],
    }
}

unsafe impl ReprStd140 for dmat4x3 {}
unsafe impl Std140ArrayElement for dmat4x3 {}

impl Deref for dmat4x3 {
    type Target = array<dvec3, 4>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for dmat4x3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for dmat4x3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("dmat4x3{:?}", &self.columns))
    }
}

/// A matrix with 4 columns and 4 rows, represented by 4 [dvec4] vectors.
///
/// # Example
///
/// ```
/// let value = std140::dmat4x4(
///     std140::dvec4(0.0, 0.0, 0.0, 1.0),
///     std140::dvec4(0.0, 0.0, 0.0, 1.0),
///     std140::dvec4(0.0, 0.0, 0.0, 1.0),
///     std140::dvec4(0.0, 0.0, 0.0, 1.0),
/// );
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct dmat4x4 {
    columns: array<dvec4, 4>,
}

impl Default for dmat4x4
{
    fn default() -> Self {
        Self::zero()
    }
}

impl dmat4x4 {
    /// Creates a new [dmat4x4] with zeros in all positions.
    pub fn zero() -> Self {
        dmat4x4(dvec4::zero(), dvec4::zero(), dvec4::zero(), dvec4::zero())
    }
}

/// Initializes a [dmat4x4][struct@dmat4x4]
///
/// # Example
///
/// See [dmat4x4][struct@dmat4x4].
pub fn dmat4x4(c0: dvec4, c1: dvec4, c2: dvec4, c3: dvec4) -> dmat4x4 {
    dmat4x4 {
        columns: array![c0, c1, c2, c3],
    }
}

unsafe impl ReprStd140 for dmat4x4 {}
unsafe impl Std140ArrayElement for dmat4x4 {}

impl Deref for dmat4x4 {
    type Target = array<dvec4, 4>;

    fn deref(&self) -> &Self::Target {
        &self.columns
    }
}

impl DerefMut for dmat4x4 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.columns
    }
}

impl fmt::Debug for dmat4x4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("dmat4x4{:?}", &self.columns))
    }
}

pub type mat2 = mat2x2;
pub type mat3 = mat3x3;
pub type mat4 = mat4x4;
pub type dmat2 = dmat2x2;
pub type dmat3 = dmat3x3;
pub type dmat4 = dmat4x4;