pub mod shadow_buffer;
pub mod spotlight;
pub mod sun_light;
pub mod point_light;

use std::ops::{Deref, DerefMut};

use getset::{CopyGetters, Getters, MutGetters, Setters};


use crate::components::ProjectionUniformData;
use crate::fast_impl_ssu;
use crate::framebuffer::Framebuffer;
use crate::game_logic::Behaviour;
use crate::game_object::GOTransform;
use crate::references::RcBox;
use crate::texture::ShaderStructUniform;
use crate::types::{Mat4, Vec3, Vec4, ArrayInto};

pub use shadow_buffer::ShadowBuffer;
pub use spotlight::*;
pub use sun_light::*;
pub use point_light::*;

crate::fast_impl_ssu! {
    #[derive(Default)]
    layout(std140) struct LightsUniformData as LightsCount {
        spotlights: i32,
        point_lights: i32,
        sun_lights: i32
    }
}

impl LightsUniformData {
    pub(crate) fn new(spotlights: i32, point_lights: i32, sun_lights: i32) -> Self {
        Self {
            spotlights: spotlights.into(),
            point_lights: point_lights.into(),
            sun_lights: sun_lights.into(),
        }
    }
}

/// Режим работы теней. Число u16 указывает размер буфера (u16 x u16).
#[derive(Copy, Clone, Debug)]
pub enum ShadowMapMode {
    /// Без теней
    None,

    /// Статичные тени.
    Static(u16),

    /// Полудинамические с отдельным буфером для динамической геометрии
    /// и отдельным для статичной.
    SemiDynamic(u16),

    /// Полностью динамические без статичного буфера.
    FullyDynamic(u16),
}

pub trait AbstractLight: Behaviour + Send + Sync {
    fn color(&self) -> Vec3;
    fn power(&self) -> f32;
    fn z_near(&self) -> f32;
    fn distance(&self) -> f32;
    fn location(&self) -> Vec3;
    fn static_shadow_buffer(&self) -> Option<&ShadowBuffer>;
    fn shadow_map_mode(&self) -> ShadowMapMode;
    fn bbox_corners(&self) -> [Vec3; 8];
    fn take_refresh_flag(&mut self) -> bool;
    fn update_transform(&mut self, transform: &GOTransform);
    fn static_shadow_framebuffers(&self) -> Vec<(Framebuffer, ProjectionUniformData)>;
    fn projections(&self) -> Vec<ProjectionUniformData>;
    fn ty(&self) -> LightType;
    fn uniform_struct(&self, shadowmap_index: i32) -> LightShaderStruct;
}

pub struct NewLight<T>
where T: AbstractLight {
    light: T
}

impl <T: AbstractLight>Deref for NewLight<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.light
    }
}

impl <T: AbstractLight>DerefMut for NewLight<T> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.light
    }
}

/// Обобщённый источник света с тенями
#[derive(Getters, Setters, MutGetters, CopyGetters, Clone)]
pub struct GenericLight {
    /// Матрица трансформации. "Пассивное" поле для сохранения местоположения и направления.
    transform: Mat4,

    /// Цвет.
    #[getset(get_copy, get_mut)]
    color: Vec3,

    /// Мощность.
    #[getset(get_copy, set)]
    power: f32,

    /// Ближняя плоскость отсечения теней.
    #[getset(get_copy, set)]
    z_near: f32,
    
    /// Расстояние источника света и дальняя плоскость отсечения теней.
    #[getset(get_copy, set)]
    distance: f32,
    
    /// Буфер статичных теней.
    static_shadow_buffer: Option<ShadowBuffer>,

    /// Режим работы теней источника света, см [`ShadowMapMode`].
    #[getset(get, set)]
    shadow_map_mode: ShadowMapMode,

    need_to_refresh_static_shadows: bool
}

impl GenericLight {
    /// Флаг потребности в обновлении буфера статичных теней.
    /// При первом вызове сбрасывается возвращает true и сбрасывается на false
    pub fn take_refresh_flag(&mut self) -> bool {
        let s = self.need_to_refresh_static_shadows;
        self.need_to_refresh_static_shadows = false;
        s
    }

    /// Буфер статичных теней.
    #[inline(always)]
    pub fn static_shadow_buffer(&self) -> Option<&ShadowBuffer>
    {
        self.static_shadow_buffer.as_ref()
    }

    /// Преобразование в структуру std140 для передачи в шейдер.
    #[inline(always)]
    pub fn uniform_struct_base(&self, projection_matrix: Mat4, shadowmap_index: i32) -> GenericLightUniform {
        GenericLightUniform {
            projection_inv  : (projection_matrix * self.transform.try_inverse().unwrap()).into(),
            color: [self.color.x, self.color.y, self.color.z, 0.0].into(),
            location: self.transform.vec4_location().into(),
            power: self.power.into(),
            z_near: self.z_near.into(),
            distance: self.distance.into(),
            shadowmap_index: shadowmap_index.into(),
        }
    }

    /// Обновление в структуру std140 для передачи в шейдер.
    #[inline(always)]
    pub fn update_transform(&mut self, transform: &GOTransform) {
        self.transform = transform.global;
    }

    #[inline(always)]
    pub fn location(&self) -> Vec3 {
        self.transform.vec3_location()
    }
}

fast_impl_ssu!{
#[derive(Default)]
layout(std140) struct GenericLightUniform {
    projection_inv: Mat4,
    color: Vec4,
    location: Vec4,
    power: f32,
    z_near: f32,
    distance: f32,
    shadowmap_index: i32
}}

impl GenericLightUniform
{
    pub fn set_shadow_map_index(&mut self, shadowmap_index: i32) {
        self.shadowmap_index = shadowmap_index.into();
    }
}

pub type Light = RcBox<dyn AbstractLight>;

#[derive(Clone, Copy)]
pub enum LightType {
    Spot,
    Sun,
    Point,
}

pub enum LightShaderStruct {
    Spot(SpotlightUniform),
    Sun(SunLightUniform),
    Point(PointLightUniform),
}

impl LightShaderStruct {

    #[inline(always)]
    pub fn base(&self) -> GenericLightUniform
    {
        match self {
            Self::Spot(li) => li.base(),
            Self::Sun(li) => li.base(),
            Self::Point(li) => li.base(),
        }
    }
    #[inline(always)]
    pub fn base_mut(&mut self) -> &mut GenericLightUniform
    {
        match self {
            Self::Spot(li) => li.base_mut(),
            Self::Sun(li) => li.base_mut(),
            Self::Point(li) => li.base_mut(),
        }
    }
}

impl LightShaderStruct
{
    pub fn ty(&self) -> LightType {
        match self {
            LightShaderStruct::Spot(_) => LightType::Spot,
            LightShaderStruct::Sun(_) => LightType::Sun,
            LightShaderStruct::Point(_) => LightType::Point,
        }
    }
}

#[test]
fn save_generic_light_uniform_header() -> std::io::Result<()>
{
    let data = format!(
        "#ifndef GENERIC_LIGHT_H\n#define GENERIC_LIGHT_H\n{}\n{}\n#endif",
        GenericLightUniform::glsl_type_name(), GenericLightUniform::structure());
    std::fs::write("./data/shaders/lighting/generic.h", &data)
}

#[test]
fn save_spotlight_uniform_header() -> std::io::Result<()>
{
    let data = format!(
        "#ifndef SPOTLIGHT_H\n#define SPOTLIGHT_H\n#include generic.h\n{}\n{}\n#endif",
        SpotlightUniform::glsl_type_name(), SpotlightUniform::structure());
    std::fs::write("./data/shaders/lighting/spotlight.h", &data)
}

#[test]
fn save_point_light_uniform_header() -> std::io::Result<()>
{
    let data = format!(
        "#ifndef POINT_LIGHT_H\n#define POINT_LIGHT_H\n#include generic.h\n{}\n{}\n#endif",
        PointLightUniform::glsl_type_name(), PointLightUniform::structure());
    std::fs::write("./data/shaders/lighting/point.h", &data)
}

#[test]
fn save_sun_light_uniform_header() -> std::io::Result<()>
{
    let data = format!(
        "#ifndef SUN_LIGHT_H\n#define SUN_LIGHT_H\n#include generic.h\n{}\n{}\n#endif",
        SunLightUniform::glsl_type_name(), SunLightUniform::structure());
    std::fs::write("./data/shaders/lighting/sun.h", &data)
}

#[test]
fn save_headers() -> std::io::Result<()>
{
    save_generic_light_uniform_header()?;
    save_spotlight_uniform_header()?;
    save_point_light_uniform_header()?;
    save_sun_light_uniform_header()
}